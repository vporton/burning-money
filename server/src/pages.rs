use actix_web::{Responder, get, HttpResponse, web};
use actix_web::http::header::CONTENT_TYPE;
use askama::Template;
use diesel::{ExpressionMethods, insert_into, RunQueryDsl};
use ethers_core::types::H160;
use serde_json::json;
use serde::Deserialize;
use crate::{Common, Config};
use crate::errors::MyError;

#[get("/aboutus")]
pub async fn about_us(config: web::Data<Config>) -> Result<impl Responder, MyError> {
    #[derive(Template)]
    #[template(path = "about.html", escape = "html")]
    struct Template {
        url_prefix: String,
    }

    Ok(Template {
        url_prefix: config.url_prefix.clone(),
    })
}

pub async fn not_found() -> actix_web::Result<HttpResponse> {
    #[derive(Template)]
    #[template(path = "error-notfound.html", escape = "html")]
    struct ErrorNotFound;
    let body = ErrorNotFound {}.render().unwrap();

    Ok(HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

#[derive(Deserialize)]
pub struct CreateAccountQuery {
    user_account: String,
}
