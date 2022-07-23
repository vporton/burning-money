use tokio_postgres_migration::Migration;
use clap::Parser;
use serde_derive::Deserialize;
use tokio_postgres::NoTls;
use server::errors::MyError;

const SCRIPTS_UP: [(&str, &str); 4] = [
    (
        "2022-06-11-034206_payments",
        include_str!("../../migrations/2022-06-11-034206_payments/up.sql"),
    ),
    (
        "2022-07-15-051941_users",
        include_str!("../../migrations/2022-07-15-051941_users/up.sql"),
    ),
    (
        "2022-07-18-132458_txs",
        include_str!("../../migrations/2022-07-18-132458_txs/up.sql"),
    ),
    (
        "2022-07-19-084330_lockfunds",
        include_str!("../../migrations/2022-07-19-084330_lockfunds/up.sql"),
    ),
];

const SCRIPTS_DOWN: [(&str, &str); 4] = [
    (
        "2022-07-19-084330_lockfunds",
        include_str!("../../migrations/2022-07-19-084330_lockfunds/down.sql"),
    ),
    (
        "2022-07-18-132458_txs",
        include_str!("../../migrations/2022-07-18-132458_txs/down.sql"),
    ),
    (
        "2022-07-15-051941_users",
        include_str!("../../migrations/2022-07-15-051941_users/down.sql"),
    ),
    (
        "2022-06-11-034206_payments",
        include_str!("../../migrations/2022-06-11-034206_payments/down.sql"),
    ),
];

enum Command {
    Up,
    Down,
    OneUp,
    OneDown,
}

#[derive(Clone, Deserialize)]
pub struct DBConfig {
    conn_string: String,
}

struct Config {
    database: DBConfig,
}

#[derive(Parser)]
struct Cli {
    #[clap(short = 'c', long = "config")]
    config: String,
    command: Command,
}

#[actix_web::main]
async fn main() -> Result<(), MyError> {
    let args = Cli::parse();

    let (mut client, connection) =
        tokio_postgres::connect(config2.database.conn_string.as_str(), NoTls).await?;
    tokio::spawn(async move { // TODO: Stop it how?
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let migration = Migration::new("migrations".to_string());
    match args.command {
        Up => migration.up(&mut client, &SCRIPTS_UP).await?,
        Down => migration.down(&mut client, &SCRIPTS_DOWN).await?,
        OneUp => migration.up(&mut client, &SCRIPTS_UP[..1]).await?,
        OneDown => migration.down(&mut client, &SCRIPTS_DOWN[..1]).await?,
    }

    Ok(())
}