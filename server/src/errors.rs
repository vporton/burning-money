use std::fmt::{Display, Formatter};
use std::io;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use actix_web::http::header::ContentType;
use askama::Template;
use ethers_core::abi::AbiError;
use lambda_web::LambdaError;
// use stripe::{RequestError, StripeError};

#[derive(Debug)]
pub struct CannotLoadOrGenerateEthereumKeyError(String);

impl CannotLoadOrGenerateEthereumKeyError {
    pub fn new(msg: String) -> Self {
        Self(msg)
    }
}

impl Display for CannotLoadOrGenerateEthereumKeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot load or generate Ethereum key: {}", self.0)
    }
}

#[derive(Debug)]
pub enum MyError {
    Template(askama::Error),
    IO(io::Error),
    DatabaseConnection(r2d2::Error),
    Database(diesel::result::Error),
    Secp256k1(secp256k1::Error),
    EthSign(ethsign::Error),
    Abi(AbiError),
    CannotLoadOrGenerateEthereumKey(CannotLoadOrGenerateEthereumKeyError),
    Toml(toml::de::Error),
    Lambda(LambdaError),
    // Stripe(StripeError),
    // StripeRequest(RequestError),
    Reqwest(reqwest::Error),
}

impl MyError {
    fn html(&self) -> String {
        match self {
            err => { // may indicate wrong UTF-8 encoding
                #[derive(Template)]
                #[template(path = "error-internal.html", escape = "html")]
                struct ErrorInternal<'a> {
                    text: &'a str,
                }
                ErrorInternal {
                    text: err.to_string().as_str(),
                }.render().unwrap()
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
            Self::DatabaseConnection(err) => write!(f, "Cannot connect to DB: {err}"),
            Self::Database(err) => write!(f, "DB error: {err}"),
            Self::Secp256k1(err) => write!(f, "(De)ciphering error: {err}"),
            Self::EthSign(err) => write!(f, "Ethereum signing error: {err}"),
            Self::Abi(err) => write!(f, "Ethereum ABI error: {err}"),
            Self::CannotLoadOrGenerateEthereumKey(err) => write!(f, "Ethereum key error: {err}"),
            Self::Toml(err) => write!(f, "INI file error: {err}"),
            Self::Lambda(err) => write!(f, "AWS Lambda error: {err}"),
            // Self::Stripe(err) => write!(f, "Stripe error: {err}"),
            // Self::StripeRequest(err) => write!(f, "Stripe request error: {err}"),
            Self::Reqwest(err) => write!(f, "Request error: {err}"),
        }
    }
}

impl ResponseError for MyError {
    fn status_code(&self) -> StatusCode {
        match self {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.html())
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

impl From<r2d2::Error> for MyError {
    fn from(value: r2d2::Error) -> Self {
        Self::DatabaseConnection(value)
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

impl From<ethsign::Error> for MyError {
    fn from(value: ethsign::Error) -> Self {
        Self::EthSign(value)
    }
}

impl From<AbiError> for MyError {
    fn from(value: AbiError) -> Self {
        Self::Abi(value)
    }
}

impl From<CannotLoadOrGenerateEthereumKeyError> for MyError {
    fn from(value: CannotLoadOrGenerateEthereumKeyError) -> Self {
        Self::CannotLoadOrGenerateEthereumKey(value)
    }
}

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
