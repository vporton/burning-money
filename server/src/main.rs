#[macro_use] extern crate diesel;
use serde_derive::Deserialize;
use std::fs;
use std::sync::Arc;
use actix_web::{App, HttpServer, web};
use actix_web::web::Data;
use env_logger::TimestampPrecision;
use clap::Parser;
use ethkey::EthAccount;
use lambda_web::{is_running_on_lambda, run_actix_on_lambda};
use errors::CannotLoadOrGenerateEthereumKeyError;
use crate::errors::MyError;
use crate::our_db_pool::{db_pool_builder, MyPool, MyDBConnectionCustomizer, MyDBConnectionManager};
use crate::pages::{about_us, create_stripe_checkout, not_found};

mod our_db_pool;
mod pages;
mod errors;
mod schema;

#[derive(Clone, Deserialize)]
pub struct Config {
    host: String,
    port: u16,
    url_prefix: String,
    secrets: SecretsConfig,
    database: DBConfig,
    stripe: StripeConfig,
}

#[derive(Clone, Deserialize)]
pub struct SecretsConfig {
    ethereum_key_file: String,
    ethereum_password: String,
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
    ethereum_key: Arc<Box<EthAccount>>,
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
    let eth_account = match EthAccount::load_or_generate(
        config.secrets.ethereum_key_file.clone(),
        config.secrets.ethereum_password.clone()
    ) {
        Ok(val) => val,
        Err(err) => Err(CannotLoadOrGenerateEthereumKeyError::new(format!("{}", err)))?, // a trouble with Sync workaround
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

    let factory = move || App::new()
        // .app_data(Data::new(config2.clone()))
        .app_data(Data::new(common.clone()))
        .service(about_us)
        .service(create_stripe_checkout)
        .service(
            actix_files::Files::new("/media", "media").use_last_modified(true),
        )
        .default_service(
            web::route().to(not_found)
        );

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