use futures::stream::StreamExt;
use std::collections::HashSet;
use diesel::OptionalExtension;
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
use diesel::{Connection, insert_into, PgConnection, RunQueryDsl, update};
use diesel::connection::{AnsiTransactionManager, TransactionManager};
use lambda_web::{is_running_on_lambda};
use rand::thread_rng;
use secp256k1::SecretKey;
use serde_json::Value;
use web3::signing::{Key, SecretKeyRef};
use web3::transports::Http;
use web3::types::{Address, BlockId, H256};
use web3::Web3;
use diesel::QueryDsl;
use diesel::ExpressionMethods;
use futures::executor::block_on;
use log::error;
use web3::api::Namespace;
use tokio_scoped::scope;
use web3::api::EthFilter;
use crate::async_db::finish_transaction;
use crate::errors::MyError;
use crate::pages::{about_us, not_found};
use crate::sql_types::TxsStatusType;
use crate::stripe::{create_payment_intent, exchange_item, stripe_public_key};
use crate::user::{user_identity, user_login, user_register};

mod pages;
mod errors;
mod stripe;
mod user;
mod async_db;
mod sql_types;
mod schema;
mod models;

static APP_USER_AGENT: &str = "CardToken seller";

#[derive(Clone, Deserialize)]
pub struct Config {
    host: String,
    port: u16,
    url_prefix: String,
    frontend_url_prefix: String,
    ethereum_network: String,
    ethereum_endpoint: String, // or Url?
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
    url: String,
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
    db: PgConnection,
    transactions_awaited: HashSet<H256>,
    // TODO: Unbounded?
    notify_transaction_tx: mpsc::UnboundedSender<()>,
    notify_transaction_rx: Arc<Mutex<mpsc::UnboundedReceiver<()>>>, // in Arc not to lock the entire struct for too long
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
    let addresses = addresses.get(&config.ethereum_network).unwrap(); // TODO: unwrap()

    let transport = Http::new(&config2.ethereum_endpoint)?;
    let transport2 = &transport.clone();

    let readonly = CommonReadonly {
        config,
        ethereum_key: Arc::new(eth_account),
        addresses: Addresses {
            // TODO: `expect()`
            token: <Address>::from_str(addresses.get("Token").expect("Can't parse addresses file").as_str().expect("Can't parse addresses file"))?,
            collateral_oracle:  <Address>::from_str(addresses.get("collateralOracle").expect("Can't parse addresses file").as_str().expect("Can't parse addresses file"))?,
        },
        web3: {
            Web3::new(transport)
        },
    };
    let (notify_transaction_tx, notify_transaction_rx) = mpsc::unbounded_channel();
    let common = Arc::new(Mutex::new(Common {
        db: PgConnection::establish(config2.database.url.as_str())?,
        transactions_awaited: HashSet::new(),
        notify_transaction_tx,
        notify_transaction_rx: Arc::new(Mutex::new(notify_transaction_rx)),
    }));

    let funds = readonly.web3.eth().balance(
         SecretKeyRef::new(&readonly.ethereum_key).address(), None).await?;
    let funds = funds.as_u64() as i64;
    { // block
        use crate::schema::global::dsl::*;
        let conn = &mut common.lock().await.db;
        web::block(|| {
            AnsiTransactionManager::begin_transaction(conn)?;
            let do_it = || {
                let v_free_funds = global.select(free_funds).for_update().first::<i64>(conn).optional()?;
                if let Some(_v_free_funds) = v_free_funds {
                    update(global).set(free_funds.eq(funds)).execute(conn)?;
                } else {
                    insert_into(global).values(free_funds.eq(funds)).execute(conn)?;
                }
                Ok::<_, MyError>(())
            };
            finish_transaction(do_it())?;
        })().await??;
    }

    let readonly = Arc::new(readonly);
    let common2 = common.clone();
    let readonly2 = readonly.clone();
    let common2 = &common2; // needed?
    let readonly2 = &readonly2; // needed?
    scope(|scope| {
        // TODO: Initialize common.transactions_awaited from DB.
        let my_loop = move || async move {
            let txs_iter = {
                use crate::schema::txs::dsl::*;
                web::block(
                    txs.filter(status.eq(TxsStatusType::Created))
                        .load(&mut common2.lock().await.db)?
                        .into_iter()
                ).await??
            };
            for tx in txs_iter {
                exchange_item(tx, common2, readonly2).await?;
            }
            loop { // TODO: Interrupt loop on exit.
                let eth = EthFilter::new(transport2.clone());
                let filter = eth.create_blocks_filter().await?;
                let mut stream = Box::pin(filter.stream(Duration::from_millis(2000))); // TODO
                let readonly = readonly2.clone();
                loop {
                    let common = common2.clone();
                    // FIXME: What to do on errors?
                    if let Some(block_hash) = stream.next().await {
                        let block_hash = block_hash?;
                        if let Some(block) = readonly.web3.eth().block(BlockId::Hash(block_hash)).await? { // TODO: `if let` correct?
                            for tx in block.transactions {
                                if common.lock().await.transactions_awaited.remove(&tx) {
                                    use crate::schema::txs::dsl::*;
                                    web::block(
                                        update(txs.filter(tx_id.eq(tx.as_bytes())))
                                            .set((status.eq(TxsStatusType::Confirmed), tx_id.eq(tx.as_bytes())))
                                            .execute(&mut common.lock().await.db)?
                                    ).await??;
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }
                let rc = { // not to lock for too long
                    let guard = common2.lock().await;
                    guard.notify_transaction_rx.clone()
                };
                rc.lock().await.recv().await;
            }
            #[allow(unreachable_code)]
            Ok::<_, MyError>(())
        };
        scope.spawn(async move {
            loop {
                if let Err(err) = my_loop().await {
                    error!("Error processing transactions: {}", err);
                }
            }
        });

        let factory = move || {
            let cors = Cors::default() // Construct CORS middleware builder
                .allowed_origin(&config2.frontend_url_prefix)
                .supports_credentials();
            let mother_hash = cookie::Key::from(config2.secrets.mother_hash.clone().as_bytes());
            App::new()
                .wrap(IdentityMiddleware::default())
                .wrap(SessionMiddleware::builder(CookieSessionStore::default(), mother_hash)
                    .cookie_secure(false) // TODO: only when testing
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

        block_on((move || async move { // FIXME: Does block_on create problems?
            if is_running_on_lambda() {
                // run_actix_on_lambda(factory).await?; // Run on AWS Lambda. // TODO
            } else {
                if let Err(err) = HttpServer::new(factory)
                    .bind((config2.host.as_str(), config2.port))?
                    .run()
                    .await
                {
                    error!("Error running HTTP server: {}", err);
                }
            }
            Ok::<_, MyError>(())
        })())?;
        Ok::<_, MyError>(())
    })?;
    Ok(())
}