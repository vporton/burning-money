use actix_web::{Responder, get, HttpResponse, web};
use askama::Template;
use diesel::{insert_into, RunQueryDsl};
use ethsign::SecretKey;
use magic_crypt::{MagicCryptTrait, new_magic_crypt};
use serde_json::json;
use crate::Config;
use crate::crypto::{random_bytes, receive_super_secret};
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

// TODO: Rewrite `@ramp-network/ramp-instant-sdk` to run this only right before payment.
#[get("/create-tmp-account")]
pub async fn create_account(q: web::Query<CreateAccountQuery>, config: web::Path<Config>) -> Result<impl Responder, MyError> {
    let secret = SecretKey::from_raw(&random_bytes())?;
    let super_secret = receive_super_secret(&*config);
    let mcrypt = new_magic_crypt!(super_secret, 256);
    let ciphered_secret = mcrypt.encrypt_to_bytes(&secret);
    let conn = config.pool.get()?;
    use crate::schema::payments::dsl::*;
    // FIXME: Make `user_account` a UNIQUE field.
    insert_into(payments)
        .values((user_account.eq(&q.user_account), temp_account_priv_key.eq(&ciphered_secret)))
        .execute(&*conn)?;
    Ok(HttpResponse::Ok().body(json!({}))) // FIXME: Content-Type
}