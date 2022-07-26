use futures::stream::StreamExt;
use std::collections::HashSet;
use tokio::sync::{mpsc, Mutex};
use serde_derive::Deserialize;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{App, cookie, HttpServer, web};
use actix_web::web::Data;
use env_logger::TimestampPrecision;
use clap::Parser;
use lambda_web::{is_running_on_lambda};
use rand::thread_rng;
use secp256k1::SecretKey;
use serde_json::Value;
use web3::signing::{Key, SecretKeyRef};
use web3::transports::Http;
use web3::types::{Address, BlockId, H256};
use web3::Web3;
use log::error;
use tokio::spawn;
use tokio_interruptible_future::interruptible;
use tokio_postgres::NoTls;
use web3::api::Namespace;
use web3::api::EthFilter;
use crate::errors::{CannotLoadDataError, MyError};
use crate::models::Tx;
use crate::pages::{about_us, not_found};
use crate::stripe::{create_payment_intent, exchange_item, lock_funds, stripe_public_key};
use crate::user::{user_identity, user_login, user_register};

mod pages;
mod errors;
mod stripe;
mod user;
mod async_db;
mod sql_types;
mod models;

static APP_USER_AGENT: &str = "CardToken seller";

#[derive(Clone, Deserialize)]
pub struct Config {
    testing: bool,
    host: String,
    port: u16,
    url_prefix: String,
    frontend_url_prefix: String,
    ethereum_network: String,
    ethereum_endpoint: String, // or Url?
    pull_ethereum: u16,
    addresses_file: String,
    secrets: SecretsConfig,
    database: DBConfig,
    stripe: StripeConfig,
}

#[derive(Clone, Deserialize)]
pub struct SecretsConfig {
    mother_hash: String,
    ethereum_key_file: String,
}

#[derive(Clone, Deserialize)]
pub struct DBConfig {
    conn_string: String,
}

#[derive(Clone, Deserialize)]
pub struct StripeConfig {
    public_key: String,
    secret_key: String,
}

#[derive(Clone)]
struct Addresses {
    token: Address,
    collateral_oracle: Address,
}

pub struct CommonReadonly {
    config: Config,
    ethereum_key: Arc<SecretKey>,
    addresses: Addresses,
    web3: Web3<Http>,
}

pub struct Common {
    db: tokio_postgres::Client,
    transactions_awaited: HashSet<H256>,
    program_finished_tx: async_channel::Sender<()>,
    // TODO: Unbounded?
    notify_transaction_tx: mpsc::UnboundedSender<()>,
    notify_transaction_rx: Arc<Mutex<mpsc::UnboundedReceiver<()>>>, // in Arc not to lock the entire struct for too long
    balance: i64,
    locked_funds: i64,
}

#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    #[clap(short = 'c', long = "config")]
    config: String,
}

#[actix_web::main]
async fn main() -> Result<(), MyError> {
    env_logger::builder()
        .format_timestamp(Some(TimestampPrecision::Millis))
        .init();

    let args = Cli::parse();

    let config: Config = toml::from_str(fs::read_to_string(args.config.as_str())?.as_str())?;

    let eth_account = {
        // let mut file = OpenOptions::new().read(true).write(true).create(true).open(config.secrets.ethereum_key_file.clone())?;
        let mut file = OpenOptions::new().read(true).write(true).create(true).open(&config.secrets.ethereum_key_file)?;
        let mut s = "".to_string();
        file.read_to_string(&mut s)?;
        let mut v = s.chars().collect::<Vec<char>>();
        loop {
            if let Some(last) = v.last() {
                if last.is_whitespace() {
                    let _ = v.pop();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        let s: String = v.into_iter().collect();
        match SecretKey::from_str(s.as_str()) {
            Ok(val) => val,
            Err(_) => {
                let result = SecretKey::new(&mut thread_rng());
                file.write(result.display_secret().to_string().as_bytes())?;
                result
            }
        }
    };
    let config2 = config.clone();

    let addresses: Value = serde_json::from_str(fs::read_to_string(config.addresses_file.as_str())?.as_str())?;
    let addresses = addresses.get(&config.ethereum_network).ok_or(CannotLoadDataError::new())?;

    let transport = Http::new(&config2.ethereum_endpoint)?;

    let readonly = CommonReadonly {
        config,
        ethereum_key: Arc::new(eth_account),
        addresses: Addresses {
            token: <Address>::from_str(addresses.get("Token").ok_or(CannotLoadDataError::new())?.as_str().ok_or(CannotLoadDataError::new())?)?,
            collateral_oracle:  <Address>::from_str(addresses.get("collateralOracle").expect("Can't parse addresses file").as_str().ok_or(CannotLoadDataError::new())?)?,
        },
        web3: {
            Web3::new(transport)
        },
    };
    let (notify_transaction_tx, notify_transaction_rx) = mpsc::unbounded_channel();
    let (client, connection) =
        tokio_postgres::connect(config2.database.conn_string.as_str(), NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let (program_finished_tx, program_finished_rx) = async_channel::bounded(1);
    let balance = readonly.web3.eth().balance(
        SecretKeyRef::new(&readonly.ethereum_key).address(), None).await?;
    let common = Arc::new(Mutex::new(Common {
        db: client,
        transactions_awaited: HashSet::new(),
        program_finished_tx,
        notify_transaction_tx,
        notify_transaction_rx: Arc::new(Mutex::new(notify_transaction_rx)),
        balance: balance.as_u64() as i64,
        locked_funds: 0,
    }));
    { // block
        let rows = common.lock().await.db
            .query("SELECT crypto_amount FROM txs WHERE status!='confirmed'", &[])
            .await?;
        for row in &rows {
            lock_funds(common.clone(), row.get(0)).await?; // takes gas into account
        }
    };

    { // restrict lock duration
        let mut common = common.lock().await;
        common.transactions_awaited = HashSet::from_iter(common.db
            .query("SELECT tx_id FROM txs WHERE status = 'submitted_to_blockchain'", &[])
            .await?
            .into_iter()
            .map(|row| H256::from_slice(row.get(0)))
        );
    }

    let readonly = Arc::new(readonly);
    let common2 = common.clone();
    let readonly2 = readonly.clone();
    let common2x = common2.clone(); // needed?
    let readonly2 = readonly2.clone(); // needed?

    let my_loop = move || {
        let common2x = common2x.clone();
        let readonly2 = readonly2.clone(); // needed?
        async move {
            let common2x = common2x.clone();
            let readonly2 = readonly2.clone(); // needed?
            let txs_iter =
                common2x.lock().await.db.query("SELECT * FROM txs WHERE status='ordered'", &[])
                    .await?
                    .into_iter();
            for tx in txs_iter {
                let tx = Tx {
                    id: tx.get("id"),
                    user_id: tx.get("user_id"),
                    eth_account: tx.get("eth_account"),
                    usd_amount: tx.get("usd_amount"),
                    crypto_amount: tx.get("crypto_amount"),
                    bid_date: tx.get("bid_date"),
                    status: tx.get("status"),
                    tx_id: tx.get("tx_id"),
                };
                exchange_item(tx, common2x.clone(), &readonly2).await?;
            }
            loop {
                let eth = EthFilter::new(readonly2.web3.transport());
                let filter = eth.create_blocks_filter().await?;
                let mut stream = Box::pin(filter.stream(Duration::from_millis(config2.pull_ethereum as u64)));
                let readonly = readonly2.clone();
                loop {
                    let common = common2x.clone();
                    // FIXME: What to do on errors?
                    if let Some(block_hash) = stream.next().await {
                        let block_hash = block_hash?;
                        if let Some(block) = readonly.web3.eth().block(BlockId::Hash(block_hash)).await? { // TODO: `if let` correct?
                            // We assume that our account can be funded by others, but only we withdraw.
                            // First remove from the locked funds, then update balance, for no races.
                            // (Balance may decrease only after having locked a sum.)
                            for tx in block.transactions {
                                let row = common.lock().await.db
                                    .query_one("SELECT id, crypto_amount FROM txs WHERE tx_id=$1", &[&tx.as_bytes()]).await?;
                                let (id, amount): (i64, i64) = (row.get(0), row.get(1));
                                { // limit lock duration
                                    let mut common = common.lock().await;
                                    if common.transactions_awaited.remove(&tx) {
                                        common.db.execute(
                                            "UPDATE txs SET status='confirmed' WHERE id=$1",
                                            &[&id]
                                        ).await?;
                                    }
                                }
                                lock_funds(common.clone(), -amount);
                            }
                            common.lock().await.balance = readonly.web3.eth().balance(
                                SecretKeyRef::new(&readonly.ethereum_key).address(), None
                            )
                                .await?.as_u64() as i64;
                        }
                    } else {
                        break;
                    }
                }
                let rc = { // not to lock for too long
                    let guard = common2x.lock().await;
                    guard.notify_transaction_rx.clone()
                };
                rc.lock().await.recv().await;
            }
            #[allow(unreachable_code)]
            Ok::<_, MyError>(())
        }
    };

    spawn(interruptible(program_finished_rx, Box::pin(async move {
        loop {
            if let Err(err) = my_loop().await {
                error!("Error processing transactions: {}", err);
            }
        }
        #[allow(unreachable_code)]
        Ok::<(), MyError>(())
    })));

    let factory = move || {
        let cors = Cors::default() // Construct CORS middleware builder
            .allowed_origin(&config2.frontend_url_prefix)
            .supports_credentials();
        let mother_hash = cookie::Key::from(config2.secrets.mother_hash.clone().as_bytes());
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::builder(CookieSessionStore::default(), mother_hash)
                .cookie_secure(!config2.testing)
                .build())
            .wrap(cors)
            // .app_data(Data::new(config2.clone()))
            .app_data(Data::new(common.clone()))
            .app_data(Data::new(readonly.clone()))
            .service(user_identity)
            .service(user_register)
            .service(user_login)
            // .service(user_logout) // TODO
            .service(about_us)
            .service(stripe_public_key)
            .service(create_payment_intent)
            .service(
                actix_files::Files::new("/media", "media").use_last_modified(true),
            )
            .default_service(
                web::route().to(not_found)
            )
    };

    if is_running_on_lambda() {
        // run_actix_on_lambda(factory).await?; // Run on AWS Lambda. // TODO
    } else {
        match HttpServer::new(factory).bind((config2.host.as_str(), config2.port)) {
            Ok(server) => if let Err(err) = server.run().await {
                error!("Error running HTTP server: {}", err);
            },
            Err(err) => error!("Error binding HTTP server: {}", err),
        }
    }
    common2.lock().await.program_finished_tx.send(()).await?;
    Ok(())
}