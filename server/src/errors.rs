use std::fmt::{Display, Formatter, write};
use std::io;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use actix_web::http::header::ContentType;
use askama::Template;

#[derive(Debug)]
pub enum MyError {
    Template(askama::Error),
    IO(io::Error),
    DatabaseConnection(r2d2::Error),
    Database(diesel::result::Error),
    Secp256k1(secp256k1::Error),
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