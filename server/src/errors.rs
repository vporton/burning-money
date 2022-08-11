use std::array::TryFromSliceError;
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::num::ParseIntError;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use actix_web::error::BlockingError;
use actix_web::http::header::{ContentType, ToStrError};
use ethers_core::abi::AbiError;
use lambda_web::LambdaError;
// use stripe::{RequestError, StripeError};
use serde::Serialize;
use tokio::task::JoinError;
use tokio_interruptible_future::InterruptError;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Not authenticated.")]
pub struct AuthenticationFailedError;

impl AuthenticationFailedError {
    pub fn new() -> Self {
        Self { }
    }
}

#[derive(Error, Debug)]
#[error("You didn't pass KYC.")]
pub struct KYCError;

impl KYCError {
    pub fn new() -> Self {
        Self { }
    }
}

#[derive(Error, Debug)]
#[error("Not enough funds.")]
pub struct NotEnoughFundsError;

impl NotEnoughFundsError {
    pub fn new() -> Self {
        Self { }
    }
}

/// Stripe misfunctions.
#[derive(Error, Debug)]
#[error("Stripe error.")]
pub struct StripeError;

impl StripeError {
    pub fn new() -> Self {
        Self { }
    }
}

#[derive(Error, Debug)]
#[error("Cannot load data.")]
pub struct CannotLoadDataError;

impl CannotLoadDataError {
    pub fn new() -> Self {
        Self { }
    }
}

#[derive(Error, Debug)]
pub enum MyErrorBase {
    Interrupt(InterruptError),
    Template(askama::Error),
    IO(io::Error),
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
    // Anyhow(MyError),
    FromHex(rustc_hex::FromHexError), // TODO: duplicate with `hex::FromHexError`?
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
    TokioPostgres(tokio_postgres::Error),
    AsyncChannelSendEmpty(async_channel::SendError<()>),
    Stripe(StripeError),
    CannotLoadData(CannotLoadDataError),
    KYC(KYCError),
    UrlParse(url::ParseError),
    ActixHeaderToStr(ToStrError),
    FromHex2(hex::FromHexError),
}

#[derive(Serialize)]
struct MyErrorJson {
    error: String,
}

impl MyErrorBase {
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
    // fn json(&self) -> MyErrorJson {
    //     match self {
    //         err => { // may indicate wrong UTF-8 encoding
    //             MyErrorJson {
    //                 error: err.to_string(),
    //             }
    //         }
    //     }
    // }
}

impl Display for MyErrorBase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Interrupt(_) => write!(f, "Fiber intrerrupted."),
            Self::Template(err) => write!(f, "Error in Askama template: {err}"),
            Self::IO(err) => write!(f, "I/O error: {err}"),
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
            // Self::Anyhow(err) => write!(f, "Error: {}", err),
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
            Self::TokioPostgres(err) => write!(f, "Postgres error: {}", err),
            Self::AsyncChannelSendEmpty(err) => write!(f, "Async send error: {}", err),
            Self::Stripe(_) => write!(f, "Stripe API failed."),
            Self::CannotLoadData(_) => write!(f, "Cannot load data."),
            Self::KYC(_) => write!(f, "KYC didn't pass."),
            Self::UrlParse(err) => write!(f, "URL parse error: {}", err),
            Self::ActixHeaderToStr(err) => write!(f, "Converting header to string: {}", err),
            Self::FromHex2(err) => write!(f, "Converting from hex: {}", err),
        }
    }
}

#[derive(Debug, Error)]
pub struct MyError {
    err: Box<anyhow::Error>,
}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&*self.err, f)
    }
}

impl From<anyhow::Error> for MyError {
    fn from(err: anyhow::Error) -> MyError {
        MyError { err: Box::new(err) }
    }
}

impl ResponseError for MyError {
    fn status_code(&self) -> StatusCode {
        if self.err.downcast_ref::<AuthenticationFailedError>().is_some() {
            StatusCode::UNAUTHORIZED
        } else if self.err.downcast_ref::<KYCError>().is_some() {
            StatusCode::UNAUTHORIZED
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
        // match self.err.downcast_ref::<MyErrorBase>().expect("Expected MyErrorBase.") {
        //     MyErrorBase::AuthenticationFailed(_) => StatusCode::UNAUTHORIZED,
        //     MyErrorBase::KYC(_) => StatusCode::UNAUTHORIZED,
        //     _ => StatusCode::INTERNAL_SERVER_ERROR,
        // }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(serde_json::to_string(&MyErrorJson {
                error: format!("{}\n{}", self.to_string(), self.err.backtrace()),
            }).unwrap())
    }
}

impl From<InterruptError> for MyErrorBase {
    fn from(value: InterruptError) -> Self {
        Self::Interrupt(value)
    }
}

impl From<askama::Error> for MyErrorBase {
    fn from(value: askama::Error) -> Self {
        Self::Template(value)
    }
}

impl From<io::Error> for MyErrorBase {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<secp256k1::Error> for MyErrorBase {
    fn from(value: secp256k1::Error) -> Self {
        Self::Secp256k1(value)
    }
}

impl From<AbiError> for MyErrorBase {
    fn from(value: AbiError) -> Self {
        Self::Abi(value)
    }
}

impl From<toml::de::Error> for MyErrorBase {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml(value)
    }
}

impl From<LambdaError> for MyErrorBase {
    fn from(value: LambdaError) -> Self {
        Self::Lambda(value)
    }
}

impl From<reqwest::Error> for MyErrorBase {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<serde_json::Error> for MyErrorBase {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<AuthenticationFailedError> for MyErrorBase {
    fn from(value: AuthenticationFailedError) -> Self {
        Self::AuthenticationFailed(value)
    }
}

impl From<rustc_hex::FromHexError> for MyErrorBase {
    fn from(value: rustc_hex::FromHexError) -> Self {
        Self::FromHex(value)
    }
}

impl From<chrono::ParseError> for MyErrorBase {
    fn from(value: chrono::ParseError) -> Self {
        Self::ParseTime(value)
    }
}

impl From<web3::Error> for MyErrorBase {
    fn from(value: web3::Error) -> Self {
        Self::Web3(value)
    }
}

impl From<web3::ethabi::Error> for MyErrorBase {
    fn from(value: web3::ethabi::Error) -> Self {
        Self::Web3Abi(value)
    }
}

impl From<web3::contract::Error> for MyErrorBase {
    fn from(value: web3::contract::Error) -> Self {
        Self::Web3Contract(value)
    }
}

impl From<TryFromSliceError> for MyErrorBase {
    fn from(value: TryFromSliceError) -> Self {
        Self::ArrayLength(value)
    }
}

impl From<NotEnoughFundsError> for MyErrorBase {
    fn from(value: NotEnoughFundsError) -> Self {
        Self::NotEnoughFunds(value)
    }
}

impl From<ParseIntError> for MyErrorBase {
    fn from(value: ParseIntError) -> Self {
        Self::ParseInt(value)
    }
}

impl From<tokio::sync::mpsc::error::SendError<()>> for MyErrorBase {
    fn from(value: tokio::sync::mpsc::error::SendError<()>) -> Self {
        Self::Send(value)
    }
}

impl From<BlockingError> for MyErrorBase {
    fn from(value: BlockingError) -> Self {
        Self::Blocking(value)
    }
}

impl From<JoinError> for MyErrorBase {
    fn from(value: JoinError) -> Self {
        Self::Join(value)
    }
}

impl From<tokio_postgres::Error> for MyErrorBase {
    fn from(value: tokio_postgres::Error) -> Self {
    Self::TokioPostgres(value)
}
}

impl From<async_channel::SendError<()>> for MyErrorBase {
    fn from(value: async_channel::SendError<()>) -> Self {
        Self::AsyncChannelSendEmpty(value)
    }
}

impl From<StripeError> for MyErrorBase {
    fn from(value: StripeError) -> Self {
        Self::Stripe(value)
    }
}

impl From<CannotLoadDataError> for MyErrorBase {
    fn from(value: CannotLoadDataError) -> Self {
        Self::CannotLoadData(value)
    }
}

impl From<KYCError> for MyErrorBase {
    fn from(value: KYCError) -> Self {
        Self::KYC(value)
    }
}

impl From<url::ParseError> for MyErrorBase {
    fn from(value: url::ParseError) -> Self {
        Self::UrlParse(value)
    }
}

impl From<ToStrError> for MyErrorBase {
    fn from(value: ToStrError) -> Self {
        Self::ActixHeaderToStr(value)
    }
}

impl From<hex::FromHexError> for MyErrorBase {
    fn from(value: hex::FromHexError) -> Self {
        Self::FromHex2(value)
    }
}

////////////////////////////////////

impl From<InterruptError> for MyError {
    fn from(value: InterruptError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<askama::Error> for MyError {
    fn from(value: askama::Error) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<io::Error> for MyError {
    fn from(value: io::Error) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<secp256k1::Error> for MyError {
    fn from(value: secp256k1::Error) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<AbiError> for MyError {
    fn from(value: AbiError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<toml::de::Error> for MyError {
    fn from(value: toml::de::Error) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

// impl From<LambdaError> for MyError {
//     fn from(value: LambdaError) -> Self {
//         MyError { err: Box::new(value.into()) }
//     }
// }

impl From<reqwest::Error> for MyError {
    fn from(value: reqwest::Error) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<serde_json::Error> for MyError {
    fn from(value: serde_json::Error) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<AuthenticationFailedError> for MyError {
    fn from(value: AuthenticationFailedError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<rustc_hex::FromHexError> for MyError {
    fn from(value: rustc_hex::FromHexError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<chrono::ParseError> for MyError {
    fn from(value: chrono::ParseError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<web3::Error> for MyError {
    fn from(value: web3::Error) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<web3::ethabi::Error> for MyError {
    fn from(value: web3::ethabi::Error) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<web3::contract::Error> for MyError {
    fn from(value: web3::contract::Error) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<TryFromSliceError> for MyError {
    fn from(value: TryFromSliceError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<NotEnoughFundsError> for MyError {
    fn from(value: NotEnoughFundsError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<ParseIntError> for MyError {
    fn from(value: ParseIntError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<tokio::sync::mpsc::error::SendError<()>> for MyError {
    fn from(value: tokio::sync::mpsc::error::SendError<()>) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<BlockingError> for MyError {
    fn from(value: BlockingError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<JoinError> for MyError {
    fn from(value: JoinError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<tokio_postgres::Error> for MyError {
    fn from(value: tokio_postgres::Error) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<async_channel::SendError<()>> for MyError {
    fn from(value: async_channel::SendError<()>) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<StripeError> for MyError {
    fn from(value: StripeError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<CannotLoadDataError> for MyError {
    fn from(value: CannotLoadDataError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<KYCError> for MyError {
    fn from(value: KYCError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<url::ParseError> for MyError {
    fn from(value: url::ParseError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<ToStrError> for MyError {
    fn from(value: ToStrError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}

impl From<hex::FromHexError> for MyError {
    fn from(value: hex::FromHexError) -> Self {
        MyError { err: Box::new(value.into()) }
    }
}
