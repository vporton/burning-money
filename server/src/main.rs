#[macro_use] extern crate diesel;
extern crate core;

use std::convert::identity;
use serde_derive::Deserialize;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::str::FromStr;
use std::sync::Arc;
use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{App, HttpServer, web};
use actix_web::cookie::Key;
use actix_web::dev::Payload::H2;
use actix_web::web::Data;
use env_logger::TimestampPrecision;
use clap::Parser;
use ethers_core::types::H256;
use lambda_web::{is_running_on_lambda, run_actix_on_lambda};
use rand::{RngCore, thread_rng};
use rand::rngs::StdRng;
use secp256k1::SecretKey;
use errors::CannotLoadOrGenerateEthereumKeyError;
use crate::errors::MyError;
use crate::our_db_pool::{db_pool_builder, MyPool, MyDBConnectionCustomizer, MyDBConnectionManager};
use crate::pages::{about_us, not_found};
use crate::stripe::{create_payment_intent, stripe_public_key};
use crate::user::{user_identity, user_login, user_register};

mod our_db_pool;
mod pages;
mod errors;
mod stripe;
mod user;
mod schema;

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
pub struct Common {
    config: Config,
    db_pool: MyPool,
    ethereum_key: Arc<secp256k1::SecretKey>,
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

    let manager = MyDBConnectionManager::new(config.database.url.clone());
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
            Err(err) => {
                let result = SecretKey::new(&mut thread_rng());
                file.write(result.display_secret().to_string().as_bytes());
                result
            }
        }
    };
    let config2 = config.clone();
    let common = Common {
        config,
        db_pool: db_pool_builder()
            .connection_customizer(Box::new(MyDBConnectionCustomizer::new()))
            .build(manager)
            .expect("Cannot connect to DB."),
        ethereum_key: Arc::new(eth_account),
    };

    let factory = move || {
        let cors = Cors::default() // Construct CORS middleware builder
            .allowed_origin(&config2.frontend_url_prefix)
            .supports_credentials();
        let mother_hash = Key::from(config2.secrets.mother_hash.clone().as_bytes());
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
        run_actix_on_lambda(factory).await?; // Run on AWS Lambda.
    } else {
        HttpServer::new(factory)
            .bind((config2.host.as_str(), config2.port))?
            .run()
            .await?;
    }
    Ok(())
}