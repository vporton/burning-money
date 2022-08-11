extern crate core;

use futures::stream::StreamExt;
use std::collections::HashSet;
use tokio::sync::{mpsc, Mutex, Semaphore};
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
use log::{debug, error, info};
use tokio::spawn;
use tokio_interruptible_future::interruptible;
use tokio_postgres::NoTls;
use web3::api::Namespace;
use web3::api::EthFilter;
use crate::errors::{CannotLoadDataError, MyError, StripeError};
use crate::kyc_sumsub::sumsub_generate_access_token;
use crate::models::Tx;
use crate::pages::{about_us, not_found};
use crate::stripe::{confirm_payment, create_payment_intent, exchange_item, fiat_to_crypto_query, lock_funds, stripe_public_key};
use crate::user::{user_email, user_identity, user_login, user_logout, user_register};

mod pages;
mod errors;
mod stripe;
mod user;
mod async_db;
mod kyc_sumsub;
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
    ethereum_endpoint: String,
    // or Url?
    pull_ethereum: u16,
    addresses_file: String,
    our_tax: f64,
    secrets: SecretsConfig,
    database: DBConfig,
    stripe: StripeConfig,
}

#[derive(Clone, Deserialize)]
pub struct SecretsConfig {
    mother_hash: String,
    ethereum_key_file: String,
    sumsub_access_token: String,
    sumsub_secret_key: String,
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
    notify_ordered_tx: mpsc::UnboundedSender<()>,
    notify_ordered_rx: Arc<Mutex<mpsc::UnboundedReceiver<()>>>,
    // notify_tx_submitted_tx: mpsc::UnboundedSender<()>,
    // notify_tx_submitted_rx: Arc<Mutex<mpsc::UnboundedReceiver<()>>>,
    // in Arc not to lock the entire struct for too long
    balance: i128,
    locked_funds: i128,
}

#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    #[clap(short = 'c', long = "config")]
    config: String,
}

/// Solve any discrepancies caused by the program improperly terminated
/// TODO: Execute it periodically.
async fn prepare_data(common: Arc<Mutex<Common>>, readonly: Arc<CommonReadonly>) -> Result<(), anyhow::Error> {
    // TODO: Remove old 'before_ordered' rows.

    // Check unfinished Stripe transactions:
    let rows = common.lock().await.db
        .query("SELECT id, payment_intent_id FROM txs WHERE status='before_ordered'", &[]).await?;
    let client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()?;
    for row in rows { // TODO: Process in parallel.
        let id: i64 = row.get(0);
        let payment_intent_id: &str = row.get(1);
        let url = format!("https://api.stripe.com/v1/payment_intents/{}", payment_intent_id);
        let res = client.post(url)
            .basic_auth::<&str, &str>(&readonly.config.stripe.secret_key, None)
            .header("Stripe-Version", "2020-08-27; server_side_confirmation_beta=v1") // needed?
            .send().await?;
        let j: Value = res.json().await?;
        if j.get("status").ok_or(StripeError::new())?.as_str().ok_or(StripeError::new())? == "confirmed" {
            common.lock().await.db
                .execute("UPDATE txs SET status='ordered' WHERE id=$1", &[&id]).await?;
            // Will submit to blockchain by `process_current()`.
        }
    }

    // TODO: Check transactions marked as 'submitted_to_blockchain', but not submitted.

    Ok(())
}

/// Starts a fiber that processes ordered payments.
async fn process_current(
    common: Arc<Mutex<Common>>,
    readonly: Arc<CommonReadonly>,
    program_finished_rx: async_channel::Receiver<()>
) -> Result<(), anyhow::Error> {
    let common2 = common.clone();
    let my_loop = move || {
        let common2 = common.clone();
        let readonly2 = readonly.clone(); // needed?
        async move {
            let common2 = common2.clone();
            let readonly2 = readonly2.clone(); // needed?
            let txs_iter =
                common2.lock().await.db.query("SELECT * FROM txs WHERE status='ordered'", &[])
                    .await?
                    .into_iter();
            for tx in txs_iter {
                let tx = Tx {
                    id: tx.get("id"),
                    user_id: tx.get("user_id"),
                    eth_account: tx.get("eth_account"),
                    usd_amount: tx.get("usd_amount"),
                    crypto_amount: i128::from_le_bytes(*<&[u8; 16]>::try_from(tx.get::<_, &[u8]>("crypto_amount"))?),
                    bid_date: tx.get("bid_date"),
                    status: tx.get("status"),
                    tx_id: tx.get("tx_id"),
                };
                exchange_item(tx, common2.clone(), &readonly2).await?;
            }
            Ok::<_, anyhow::Error>(())
        }
    };

    let common3 = common2.clone();
    spawn(interruptible(program_finished_rx, Box::pin(async move {
        let common = &common3;
        loop {
            if let Err(err) = my_loop().await {
                error!("Error processing transactions: {}\n{}", err, err.backtrace());
            }
            let rc = { // not to lock for too long
                let guard = common.lock().await;
                guard.notify_ordered_rx.clone()
            };
            rc.lock().await.recv().await;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    })));

    Ok(())
}

async fn process_blocks(
        common: Arc<Mutex<Common>>,
        readonly: Arc<CommonReadonly>,
        program_finished_rx: async_channel::Receiver<()>
) -> Result<(), anyhow::Error> {
    let common3 = common.clone();
    let readonly2 = readonly.clone();

    let my_loop = move || {
        let common2 = common.clone();
        let readonly2 = readonly.clone(); // needed?
        async move {
            let common2 = common2.clone();
            let readonly2 = readonly2.clone(); // needed?
            let readonly2y = &readonly2;
            loop {
                let config3 = &readonly2y.config;
                let eth = EthFilter::new(readonly2.web3.transport());
                let filter = eth.create_blocks_filter().await?;
                let mut stream = Box::pin(filter.stream(Duration::from_millis(config3.pull_ethereum as u64)));
                let readonly = readonly2.clone();
                loop {
                    let common = common2.clone();
                    if let Some(block_hash) = stream.next().await {
                        let block_hash = block_hash?;
                        if let Some(block) = readonly.web3.eth().block(BlockId::Hash(block_hash)).await? {
                            // We assume that our account can be funded by others, but only we withdraw.
                            // First remove from the locked funds, then update balance, for no races.
                            // (Balance may decrease only after having locked a sum.)
                            if let (Some(number), false) = (block.number, block.transactions.is_empty()) {
                                debug!(
                                    "BLOCK #{} txs: {}",
                                    number,
                                    block.transactions.iter().map(|t| t.to_string() + " ").collect::<String>());
                                debug!(
                                    "AWAITED: {}",
                                    common.lock().await.transactions_awaited.iter().map(|t| t.to_string() + " ").collect::<String>());
                            }
                            for tx in block.transactions {
                                let row = common.lock().await.db
                                    .query_opt("SELECT id, crypto_amount FROM txs WHERE tx_id=$1", &[&tx.as_bytes()]).await?;
                                if let Some(row) = row {
                                    let (id, amount): (i64, &[u8]) = (row.get(0), row.get(1));
                                    let amount = i128::from_le_bytes(<[u8; 16]>::try_from(amount)?);
                                    { // limit lock duration
                                        let mut common = common.lock().await;
                                        if common.transactions_awaited.remove(&tx) {
                                            common.db.execute(
                                                "UPDATE txs SET status='confirmed' WHERE id=$1",
                                                &[&id],
                                            ).await?;
                                        }
                                    }
                                    lock_funds(common.clone(), -amount).await?;
                                }
                            }
                            // TODO: The following is a lock for an extended period of time.
                            common.lock().await.balance = readonly.web3.eth().balance(
                                SecretKeyRef::new(&readonly.ethereum_key).address(), None,
                            )
                                .await?.as_u128() as i128;
                        }
                    } else {
                        break;
                    }
                }
            }
            #[allow(unreachable_code)]
            Ok::<_, anyhow::Error>(())
        }
    };

    spawn(interruptible(program_finished_rx, Box::pin(async move {
        loop {
            if let Err(err) = my_loop().await {
                error!("Error processing blocks: {}", err);
            }
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    })));

    // FIXME: Run this only after waiting for the first block starts (for no races).
    { // block
        let rows =
            common3.lock().await.db.query("SELECT tx_id FROM txs WHERE status='submitted_to_blockchain'", &[])
                .await?;
        for row in rows {
            let tx = H256::from_slice(row.get(0));
            let common3 = common3.clone();
            let readonly2 = readonly2.clone();
            spawn((move || async move {
                let _semaphore = Semaphore::new(10); // TODO: Make configurable.
                let receipt = readonly2.clone().web3.eth().transaction_receipt(tx).await?;
                if receipt.is_some() {
                    common3.lock().await.transactions_awaited.remove(&tx);
                    let conn = &common3.lock().await.db;
                    conn.execute(
                        "UPDATE txs SET status='confirmed' WHERE tx_id=$1",
                        &[&tx.as_bytes()]
                    ).await?;
                }
                Ok::<_, anyhow::Error>(())
            })());
        }
    }

    Ok(())
}

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::builder()
        .format_timestamp(Some(TimestampPrecision::Millis))
        .init();

    let args = Cli::parse();

    let config: Config = toml::from_str(fs::read_to_string(args.config.as_str())?.as_str())?;

    let eth_account = {
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
    info!("Ethereum address: {}", SecretKeyRef::new(&eth_account).address());
    let config2 = config.clone();

    let addresses: Value = serde_json::from_str(fs::read_to_string(config.addresses_file.as_str())?.as_str())?;
    let addresses = addresses.get(&config.ethereum_network).ok_or(CannotLoadDataError::new())?;

    let transport = Http::new(&config2.ethereum_endpoint)?;

    let readonly = CommonReadonly {
        config,
        ethereum_key: Arc::new(eth_account),
        addresses: Addresses {
            token: <Address>::from_str(addresses.get("Token").ok_or(CannotLoadDataError::new())?.as_str().ok_or(CannotLoadDataError::new())?)?,
            collateral_oracle: <Address>::from_str(addresses.get("collateralOracle").ok_or(CannotLoadDataError::new())?.as_str().ok_or(CannotLoadDataError::new())?)?,
        },
        web3: {
            Web3::new(transport)
        },
    };
    let (notify_ordered_tx, notify_ordered_rx) = mpsc::unbounded_channel();
    // let (notify_tx_submitted_tx, notify_tx_submitted_rx) = mpsc::unbounded_channel();
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
    info!("Ethereum {} balance: {balance}", SecretKeyRef::new(&readonly.ethereum_key).address());
    let common = Arc::new(Mutex::new(Common {
        db: client,
        transactions_awaited: HashSet::new(),
        program_finished_tx,
        notify_ordered_tx,
        notify_ordered_rx: Arc::new(Mutex::new(notify_ordered_rx)),
        // notify_tx_submitted_tx,
        // notify_tx_submitted_rx: Arc::new(Mutex::new(notify_tx_submitted_rx)),
        balance: balance.as_u128() as i128,
        locked_funds: 0,
    }));
    { // block
        let rows = common.lock().await.db
            .query("SELECT crypto_amount FROM txs WHERE status!='confirmed'", &[])
            .await?;
        for row in &rows {
            let bytes: &[u8] = row.get(0);
            let amount = i128::from_le_bytes(<[u8; 16]>::try_from(bytes)?);
            lock_funds(common.clone(), amount).await?; // takes gas into account
        }
    };

    { // restrict lock duration
        let mut common = common.lock().await;
        common.transactions_awaited = HashSet::from_iter(common.db
            .query("SELECT tx_id FROM txs WHERE status = 'submitted_to_blockchain'", &[])
            .await?
            .into_iter()
            .filter_map(|row| if let Some(tx) = row.get(0) {
                Some(H256::from_slice(tx))
            } else {
                None
            })
        );
    }

    let readonly = Arc::new(readonly);
    let common2 = common.clone();
    let readonly2 = readonly.clone();
    let common2x = common2.clone(); // needed?
    let readonly2 = readonly2.clone(); // needed?

    prepare_data(common2x.clone(), readonly2.clone()).await?;
    // TODO: Possible races between process_current() and process_blocks().
    process_current(common2x.clone(), readonly2.clone(), program_finished_rx.clone()).await?;
    process_blocks(common2x.clone(), readonly2.clone(), program_finished_rx).await?;

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
            .service(user_email)
            .service(user_register)
            .service(user_login)
            .service(user_logout)
            .service(about_us)
            .service(stripe_public_key)
            .service(create_payment_intent)
            .service(confirm_payment)
            .service(sumsub_generate_access_token)
            .service(fiat_to_crypto_query)
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