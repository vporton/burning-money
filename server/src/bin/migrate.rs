use std::fmt::{Display, Formatter};
use std::fs;
use std::str::FromStr;
use tokio_postgres_migration::Migration;
use clap::Parser;
use serde::de::StdError;
use serde_derive::Deserialize;
use tokio_postgres::NoTls;
use crate::MyError::{DeserailizeToml, Postgres};

#[derive(Debug)]
struct CommandError;

impl StdError for CommandError { }

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wrong command")
    }
}

#[derive(Debug)]
enum MyError {
    Postgres(tokio_postgres::Error),
    DeserailizeToml(toml::de::Error),
    IO(std::io::Error),
    Command(CommandError),
}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Postgres(err) => write!(f, "Postgres error: {err}"),
            Self::DeserailizeToml(err) => write!(f, "Error reading config: {err}"),
            Self::IO(err) => write!(f, "I/O error: {err}"),
            Self::Command(_) => write!(f, "Wrong command"),
        }
    }
}

impl From<tokio_postgres::Error> for MyError {
    fn from(value: tokio_postgres::Error) -> Self {
        Postgres(value)
    }
}

impl From<toml::de::Error> for MyError {
    fn from(value: toml::de::Error) -> Self {
        DeserailizeToml(value)
    }
}

impl From<std::io::Error> for MyError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<CommandError> for MyError {
    fn from(value: CommandError) -> Self {
        Self::Command(value)
    }
}

const SCRIPTS_UP: [(&str, &str); 2] = [
    (
        "2022-07-15-051941_users",
        include_str!("../../migrations/2022-07-15-051941_users/up.sql"),
    ),
    (
        "2022-07-18-132458_txs",
        include_str!("../../migrations/2022-07-18-132458_txs/up.sql"),
    ),
];

const SCRIPTS_DOWN: [(&str, &str); 2] = [
    (
        "2022-07-18-132458_txs",
        include_str!("../../migrations/2022-07-18-132458_txs/down.sql"),
    ),
    (
        "2022-07-15-051941_users",
        include_str!("../../migrations/2022-07-15-051941_users/down.sql"),
    ),
];

enum Command {
    Up,
    Down,
    OneUp,
    OneDown,
    Redo,
}

impl FromStr for Command {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            "oneup" => Ok(Self::OneUp),
            "onedown" => Ok(Self::OneDown),
            "redo" => Ok(Self::Redo),
            _ => Err(CommandError),
        }
    }
}

#[derive(Deserialize)]
pub struct DBConfig {
    conn_string: String,
}

#[derive(Deserialize)]
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
async fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();
    let config: Config = toml::from_str(fs::read_to_string(args.config.as_str())?.as_str())?;

    let (mut client, connection) =
        tokio_postgres::connect(config.database.conn_string.as_str(), NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let migration = Migration::new("migrations".to_string());
    match args.command {
        Command::Up => migration.up(&mut client, &SCRIPTS_UP).await?,
        Command::Down => migration.down(&mut client, &SCRIPTS_DOWN).await?,
        Command::OneUp => migration.up(&mut client, &SCRIPTS_UP[..1]).await?,
        Command::OneDown => migration.down(&mut client, &SCRIPTS_DOWN[..1]).await?,
        Command::Redo => {
            migration.down(&mut client, &SCRIPTS_DOWN[..1]).await?;
            migration.up(&mut client, &SCRIPTS_UP[..1]).await?;
        },
    }

    Ok(())
}