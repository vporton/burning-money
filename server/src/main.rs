#[macro_use] extern crate diesel;
use serde_derive::Deserialize;
use std::fs;
use actix_web::{App, HttpServer, web};
use actix_web::web::Data;
use env_logger::TimestampPrecision;
use clap::Parser;
use lambda_web::{is_running_on_lambda, LambdaError, run_actix_on_lambda};
use crate::our_db_pool::{db_pool_builder, MyPool, MyDBConnectionCustomizer, MyDBConnectionManager};
use crate::pages::{about_us, initiate_payment, not_found};

mod our_db_pool;
mod pages;
mod errors;
mod crypto;
mod schema;

#[derive(Clone, Deserialize)]
pub struct Config {
    host: String,
    port: u16,
    url_prefix: String,
    super_secret_file: String,
    database: DBConfig,
}

#[derive(Clone, Deserialize)]
pub struct DBConfig {
    url: String,
}

#[derive(Clone)]
pub struct Common {
    db_pool: MyPool,
}

#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    #[clap(short = 'c', long = "config")]
    config: String,
}

#[actix_web::main]
async fn main() -> Result<(), LambdaError> {
    env_logger::builder()
        .format_timestamp(Some(TimestampPrecision::Millis))
        .init();

    let args = Cli::parse();

    let config: Config = toml::from_str(fs::read_to_string(args.config.as_str())?.as_str())?;

    let manager = MyDBConnectionManager::new(config.database.url.clone());
    let common = Common {
        db_pool: db_pool_builder()
            .connection_customizer(Box::new(MyDBConnectionCustomizer::new()))
            .build(manager)
            .expect("Cannot connect to DB."),
    };

    let config2 = config.clone();
    let factory = move || App::new()
        .app_data(Data::new(config2.clone()))
        .app_data(Data::new(common.clone()))
        .service(about_us)
        .service(initiate_payment)
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
            .bind((config.host.as_str(), config.port))?
            .run()
            .await?;
    }
    Ok(())
}