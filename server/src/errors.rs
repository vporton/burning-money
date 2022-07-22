use std::array::TryFromSliceError;
use std::fmt::{Display, Formatter};
use std::io;
use std::num::ParseIntError;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use actix_web::error::BlockingError;
use actix_web::http::header::ContentType;
use diesel::ConnectionError;
use ethers_core::abi::AbiError;
use lambda_web::LambdaError;
// use stripe::{RequestError, StripeError};
use serde::Serialize;
use tokio::task::JoinError;

// #[derive(Debug)]
// pub struct CannotLoadOrGenerateEthereumKeyError(String);
//
// impl CannotLoadOrGenerateEthereumKeyError {
//     // TODO
//     pub fn new(msg: String) -> Self {
//         Self(msg)
//     }
// }
//
// impl Display for CannotLoadOrGenerateEthereumKeyError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Cannot load or generate Ethereum key: {}", self.0)
//     }
// }

#[derive(Debug)]
pub struct AuthenticationFailedError;

impl AuthenticationFailedError {
    pub fn new() -> Self {
        Self { }
    }
}

impl Display for AuthenticationFailedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wrong password.")
    }
}

#[derive(Debug)]
pub struct NotEnoughFundsError;

impl NotEnoughFundsError {
    pub fn new() -> Self {
        Self { }
    }
}

impl Display for NotEnoughFundsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Not enough funds.")
    }
}

#[derive(Debug)]
pub enum MyError {
    Template(askama::Error),
    IO(io::Error),
    DbConnection(ConnectionError),
    Database(diesel::result::Error),
    Secp256k1(secp256k1::Error),
    Abi(AbiError),
    // CannotLoadOrGenerateEthereumKey(CannotLoadOrGenerateEthereumKeyError),
    Toml(toml::de::Error),
    Lambda(LambdaError),
    // Stripe(StripeError),
    // StripeRequest(RequestError),
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
    AuthenticationFailed(AuthenticationFailedError),
    Anyhow(anyhow::Error),
    FromHex(rustc_hex::FromHexError),
    ParseTime(chrono::ParseError),
    Web3(web3::Error),
    Web3Abi(web3::ethabi::Error),
    Web3Contract(web3::contract::Error),
    ArrayLength(TryFromSliceError),
    NotEnoughFunds(NotEnoughFundsError),
    ParseInt(ParseIntError),
    Send(tokio::sync::mpsc::error::SendError<()>),
    Blocking(BlockingError),
    Join(JoinError),
}

#[derive(Serialize)]
struct MyErrorJson {
    error: String,
}

impl MyError {
    // fn html(&self) -> String {
    //     match self {
    //         err => { // may indicate wrong UTF-8 encoding
    //             #[derive(Template)]
    //             #[template(path = "error-internal.html", escape = "html")]
    //             struct ErrorInternal<'a> {
    //                 text: &'a str,
    //             }
    //             ErrorInternal {
    //                 text: err.to_string().as_str(),
    //             }.render().unwrap()
    //         }
    //     }
    // }
    fn json(&self) -> MyErrorJson {
        match self {
            err => { // may indicate wrong UTF-8 encoding
                MyErrorJson {
                    error: err.to_string(),
                }
            }
        }
    }
}

impl std::error::Error for MyError { }

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Template(err) => write!(f, "Error in Askama template: {err}"),
            Self::IO(err) => write!(f, "I/O error: {err}"),
            Self::DbConnection(err) => write!(f, "Cannot connect to DB: {err}"),
            Self::Database(err) => write!(f, "DB error: {err}"),
            Self::Secp256k1(err) => write!(f, "(De)ciphering error: {err}"),
            // Self::EthSign(err) => write!(f, "Ethereum signing error: {err}"),
            Self::Abi(err) => write!(f, "Ethereum ABI error: {err}"),
            // Self::CannotLoadOrGenerateEthereumKey(err) => write!(f, "Ethereum key error: {err}"),
            Self::Toml(err) => write!(f, "INI file error: {err}"),
            Self::Lambda(err) => write!(f, "AWS Lambda error: {err}"),
            // Self::Stripe(err) => write!(f, "Stripe error: {err}"),
            // Self::StripeRequest(err) => write!(f, "Stripe request error: {err}"),
            Self::Reqwest(err) => write!(f, "Request error: {err}"),
            Self::Json(err) => write!(f, "JSON error: {err}"),
            Self::AuthenticationFailed(_) => write!(f, "Authentication failed."),
            Self::Anyhow(err) => write!(f, "Error: {}", err),
            Self::FromHex(_err) => write!(f, "Error converting from hex"),
            Self::ParseTime(err) => write!(f, "Parsing time: {}", err),
            Self::Web3(err) => write!(f, "Web3 error: {}", err),
            Self::Web3Abi(err) => write!(f, "Web3 ABI error: {}", err),
            Self::Web3Contract(err) => write!(f, "Web3 contract error: {}", err),
            Self::ArrayLength(err) => write!(f, "Array length error: {}", err),
            Self::NotEnoughFunds(_) => write!(f, "Not enough funds."),
            Self::ParseInt(_) => write!(f, "Cannot parse integer."),
            Self::Send(_) => write!(f, "Send () error."),
            Self::Blocking(_) => write!(f, "Blocking error."),
            Self::Join(_) => write!(f, "Join error."),
        }
    }
}

impl ResponseError for MyError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::AuthenticationFailed(_) => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR, // TODO
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(serde_json::to_string(&self.json()).unwrap())
    }
}

impl From<askama::Error> for MyError {
    fn from(value: askama::Error) -> Self {
        Self::Template(value)
    }
}

impl From<io::Error> for MyError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<ConnectionError> for MyError {
    fn from(value: ConnectionError) -> Self {
        Self::DbConnection(value)
    }
}

impl From<diesel::result::Error> for MyError {
    fn from(value: diesel::result::Error) -> Self {
        Self::Database(value)
    }
}

impl From<secp256k1::Error> for MyError {
    fn from(value: secp256k1::Error) -> Self {
        Self::Secp256k1(value)
    }
}

impl From<AbiError> for MyError {
    fn from(value: AbiError) -> Self {
        Self::Abi(value)
    }
}

// impl From<CannotLoadOrGenerateEthereumKeyError> for MyError {
//     fn from(value: CannotLoadOrGenerateEthereumKeyError) -> Self {
//         Self::CannotLoadOrGenerateEthereumKey(value)
//     }
// }

impl From<toml::de::Error> for MyError {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml(value)
    }
}

impl From<LambdaError> for MyError {
    fn from(value: LambdaError) -> Self {
        Self::Lambda(value)
    }
}

// impl From<StripeError> for MyError {
//     fn from(value: StripeError) -> Self {
//         if let StripeError::Stripe(request) = value {
//             Self::StripeRequest(request)
//         } else {
//             Self::Stripe(value)
//         }
//     }
// }

impl From<reqwest::Error> for MyError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<serde_json::Error> for MyError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<AuthenticationFailedError> for MyError {
    fn from(value: AuthenticationFailedError) -> Self {
        Self::AuthenticationFailed(value)
    }
}

impl From<anyhow::Error> for MyError {
    fn from(value: anyhow::Error) -> Self {
        Self::Anyhow(value)
    }
}

impl From<rustc_hex::FromHexError> for MyError {
    fn from(value: rustc_hex::FromHexError) -> Self {
        Self::FromHex(value)
    }
}

impl From<chrono::ParseError> for MyError {
    fn from(value: chrono::ParseError) -> Self {
        Self::ParseTime(value)
    }
}

impl From<web3::Error> for MyError {
    fn from(value: web3::Error) -> Self {
        Self::Web3(value)
    }
}

impl From<web3::ethabi::Error> for MyError {
    fn from(value: web3::ethabi::Error) -> Self {
        Self::Web3Abi(value)
    }
}

impl From<web3::contract::Error> for MyError {
    fn from(value: web3::contract::Error) -> Self {
        Self::Web3Contract(value)
    }
}

impl From<TryFromSliceError> for MyError {
    fn from(value: TryFromSliceError) -> Self {
        Self::ArrayLength(value)
    }
}

impl From<NotEnoughFundsError> for MyError {
    fn from(value: NotEnoughFundsError) -> Self {
        Self::NotEnoughFunds(value)
    }
}

impl From<ParseIntError> for MyError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseInt(value)
    }
}

impl From<tokio::sync::mpsc::error::SendError<()>> for MyError {
    fn from(value: tokio::sync::mpsc::error::SendError<()>) -> Self {
        Self::Send(value)
    }
}

impl From<BlockingError> for MyError {
    fn from(value: BlockingError) -> Self {
        Self::Blocking(value)
    }
}

impl From<JoinError> for MyError {
    fn from(value: JoinError) -> Self {
        Self::Join(value)
    }
}