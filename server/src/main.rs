use futures::stream::StreamExt;
use std::collections::HashSet;
use diesel::OptionalExtension;
use tokio::sync::Mutex;
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
use lambda_web::{is_running_on_lambda, run_actix_on_lambda};
use rand::thread_rng;
use secp256k1::SecretKey;
use serde_json::Value;
use web3::signing::{Key, SecretKeyRef};
use web3::transports::Http;
use web3::types::{Address, BlockId, H256};
use web3::Web3;
use diesel::QueryDsl;
use diesel::ExpressionMethods;
use web3::api::{Eth, Namespace};
use tokio::spawn;
use web3::api::EthFilter;
use crate::errors::MyError;
use crate::pages::{about_us, not_found};
use crate::sql_types::TxsStatusType;
use crate::stripe::{create_payment_intent, stripe_public_key};
use crate::user::{user_identity, user_login, user_register};

mod pages;
mod errors;
mod stripe;
mod user;
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

#[derive(Clone)]
pub struct Common {
    config: Config,
    db: Arc<Mutex<PgConnection>>,
    ethereum_key: Arc<secp256k1::SecretKey>,
    addresses: Addresses,
    web3: Web3<Http>,
    transactions_awaited: Arc<Mutex<HashSet<H256>>>,
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
        while v.last().unwrap().is_whitespace() { // FIXME: unwrap()
            let _ = v.pop();
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
    let transport2 = transport.clone();
    let common = Common {
        config,
        db: Arc::new(Mutex::new(PgConnection::establish(config2.database.url.as_str())?)),
        ethereum_key: Arc::new(eth_account),
        addresses: Addresses {
            // TODO: `expect()`
            token: <Address>::from_str(addresses.get("Token").expect("Can't parse addresses file").as_str().expect("Can't parse addresses file"))?,
            collateral_oracle:  <Address>::from_str(addresses.get("collateralOracle").expect("Can't parse addresses file").as_str().expect("Can't parse addresses file"))?,
        },
        web3: {
            Web3::new(transport)
        },
        transactions_awaited: Arc::new(Mutex::new(HashSet::new())),
    };
    let web32 = common.web3.clone(); // TODO: right way?

    let funds = common.web3.eth().balance(
         SecretKeyRef::new(&common.ethereum_key).address(), None).await?;
    let funds = funds.as_u64() as i64;
    { // block
        use crate::schema::global::dsl::*;
        // FIXME: transaction
        let v_free_funds = global.select(free_funds).for_update().first::<i64>(&mut *common.db.lock().await).optional()?;
        if let Some(_v_free_funds) = v_free_funds {
            update(global).set(free_funds.eq(funds)).execute(&mut *common.db.lock().await)?;
        } else {
            insert_into(global).values(free_funds.eq(funds)).execute(&mut *common.db.lock().await)?;
        }
    }

    let transactions_awaited2 = common.transactions_awaited.clone();
    let db2 = common.db.clone();
    spawn((move || async move {
        loop { // TODO: Interrupt loop on exit.
            // FIXME: Make pauses.
            let eth = EthFilter::new(transport2.clone());
            let filter = eth.create_blocks_filter().await?;
            let mut stream = Box::pin(filter.stream(Duration::from_millis(2000))); // TODO
            loop {
                let transactions_awaited = transactions_awaited2.clone();
                // FIXME: What to do on errors?
                if let Some(block_hash) = stream.next().await {
                    let block_hash = block_hash?;
                    if let Some(block) = web32.eth().block(BlockId::Hash(block_hash)).await? { // TODO: `if let` correct?
                        for tx in block.transactions {
                            if transactions_awaited.lock().await.remove(&tx) {
                                use crate::schema::txs::dsl::*;
                                update(txs.filter(tx_id.eq(tx.as_bytes())))
                                    .set((status.eq(TxsStatusType::Confirmed), tx_id.eq(tx.as_bytes())))
                                    .execute(&mut *db2.lock().await)?;
                            }
                        }
                    }
                }
            }
            // TODO
        }
        #[allow(unreachable_code)]
        Ok::<_, MyError>(())
    })());

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
        run_actix_on_lambda(factory).await?; // Run on AWS Lambda. // TODO
    } else {
        HttpServer::new(factory)
            .bind((config2.host.as_str(), config2.port))?
            .run()
            .await?;
    }
    Ok(())
}