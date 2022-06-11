use actix_web::{Responder, get, HttpResponse, web};
use askama::Template;
use diesel::{ExpressionMethods, insert_into, RunQueryDsl};
use ethsign::keyfile::Crypto;
use ethsign::SecretKey;
use ethers_core::abi::AbiDecode;
use ethers_core::types::H160;
use serde_json::json;
use serde::Deserialize;
use crate::{Common, Config};
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

#[derive(Deserialize)]
pub struct CreateAccountQuery {
    user_account: String,
}

// TODO: Rewrite `@ramp-network/ramp-instant-sdk` to run this only right before payment.
#[get("/create-tmp-account")]
pub async fn create_account(q: web::Query<CreateAccountQuery>, common: web::Data<Common>, config: web::Data<Config>) -> Result<impl Responder, MyError> {
    let secret = SecretKey::from_raw(&random_bytes())?;
    let super_secret = receive_super_secret(&*config)?;

    // TOOD: Move Protected to `crypto`
    let ciphered_secret: Crypto = secret.to_crypto(&super_secret, 1)?;

    // let ciphered_secret: Ciphertext = encrypt(secret, &super_secret, CiphertextVersion::V2).expect("encryption shouldn't fail");
    let conn = common.db_pool.get()?;
    let v_user_account = H160::decode_hex(q.user_account.clone())?.0;
    use crate::schema::payments::dsl::*;
    // FIXME: Make `user_account` a UNIQUE field.
    insert_into(payments)
        .values((user_account.eq(&v_user_account as &[u8]), temp_account_priv_key.eq(ciphered_secret.ciphertext.0)))
        // .values((user_account.eq(<&[u8]>::from(&v_user_account)), temp_account_priv_key.eq(ciphered_secret.into())))
        .execute(&*conn)?;
    Ok(HttpResponse::Ok().body(serde_json::to_vec(&json!({})).unwrap())) // FIXME: Content-Type
}