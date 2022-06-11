use actix_web::{Responder, get, HttpResponse, web};
use askama::Template;
use clap::lazy_static::lazy_static;
use diesel::{insert_into, RunQueryDsl};
use ethsign::SecretKey;
use crate::Config;
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

struct CreateAccountQuery {
    user_account: String,
}

#[get("/create-tmp-account")]
pub async fn create_account(q: web::Query<CreateAccountQuery>, config: web::Path<[u8; 32]>) -> Result<impl Responder, MyError> {
    let secret = SecretKey::from_raw(&random_bytes())?;
    let super_secret = receive_super_secret(&*config);
    let mcrypt = new_magic_crypt!(super_secret, 256);
    let ciphered_secret = mcrypt.encrypt_to_bytes(secret);
    insert_into(payments)
        .values(user_account.eq(q.user_account), temp_account_priv_key.eq(ciphered_secret))
        .execute(??)?;
    Ok(HttpResponse::Ok().body(json!{}))
}