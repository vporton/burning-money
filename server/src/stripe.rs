use diesel::{Connection, ExpressionMethods, update};
use web3::types::*;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use actix_identity::Identity;
use actix_web::{get, post, Responder, web, HttpResponse};
use actix_web::http::header::CONTENT_TYPE;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::{insert_into, RunQueryDsl};
// use stripe::{CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession, CreateCheckoutSessionLineItems, CreatePrice, CreateProduct, Currency, IdOrCreate, Price, Product};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web3::contract::{Contract, Options};
use diesel::QueryDsl;
use tokio::sync::Mutex;
use crate::{Common, CommonReadonly, MyError};
use crate::errors::{AuthenticationFailedError, NotEnoughFundsError};
use crate::sql_types::TxsStatusType;

// We follow https://stripe.com/docs/payments/finalize-payments-on-the-server

#[derive(Deserialize)]
pub struct CreateStripeCheckout {
    fiat_amount: f64,
}

#[get("/stripe-pubkey")]
pub async fn stripe_public_key(readonly: web::Data<CommonReadonly>) -> impl Responder {
    HttpResponse::Ok().body(readonly.config.stripe.public_key.clone())
}

#[post("/create-payment-intent")]
pub async fn create_payment_intent(
    q: web::Query<CreateStripeCheckout>,
    ident: Identity,
    common: web::Data<Arc<Mutex<Common>>>,readonly: web::Data<CommonReadonly>
) -> Result<impl Responder, MyError> {
    { // block
        use crate::schema::users::dsl::*;
        let v_passed_kyc: bool = users.filter(id.eq(ident.id()?.parse::<i64>()?))
            .select(passed_kyc)
            .get_result(&mut common.lock().await.db)?;
        if !v_passed_kyc {
            return Err(AuthenticationFailedError::new().into()); // TODO: more specific error
        }
    }
    let client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()?;
    let mut params = HashMap::new();
    let fiat_amount = q.fiat_amount.to_string();
    let user_id = ident.id()?;
    params.insert("amount", fiat_amount.as_str());
    params.insert("currency", "usd");
    params.insert("automatic_payment_methods[enabled]", "true");
    params.insert("secret_key_confirmation", "required");
    params.insert("metadata[user]", user_id.as_str());
    let res = client.post("https://api.stripe.com/v1/payment_intents")
        .basic_auth::<&str, &str>(&readonly.config.stripe.secret_key, None)
        .header("Stripe-Version", "2020-08-27; server_side_confirmation_beta=v1")
        .form(&params)
        .send().await?;
    // FIXME: On error (e.g. fiat_amount<100) return JSON error.
    #[derive(Deserialize, Serialize)]
    struct Data {
        id: String,
        client_secret: String,
    }
    let data: Data = serde_json::from_slice(res.bytes().await?.as_ref())?;
    Ok(web::Json(data))
}

async fn finalize_payment(payment_intent_id: &str, readonly: &Arc<CommonReadonly>) -> Result<(), MyError> {
    let client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()?;
    let url = format!("https://api.stripe.com/v1/payment_intents/{}/confirm", payment_intent_id);
    client.post(url)
        .basic_auth::<&str, &str>(&readonly.config.stripe.secret_key, None)
        .send().await?;
    Ok(())
}

async fn lock_funds(common: &Arc<Mutex<Common>>, amount: i64) -> Result<(), MyError> {
    let conn = &mut common.lock().await.db;
    conn.transaction::<_, MyError, _>(|conn| {
        use crate::schema::global::dsl::*;
        let v_free_funds = global.select(free_funds).for_update().get_result::<i64>(conn)?;
        if amount >= v_free_funds { // FIXME: Take gas into account.
            return Err(NotEnoughFundsError::new().into());
        }
        update(global).set(free_funds.eq(free_funds - amount)).execute(conn)?;
        Ok(())
    })?;
    Ok(())
}

// It returns the Ethereum transaction (probably, yet not confirmed).
async fn do_exchange(readonly: &Arc<CommonReadonly>, crypto_account: Address, bid_date: DateTime<Utc>, crypto_amount: i64)
    -> Result<H256, MyError>
{
    let token =
        Contract::from_json(
            readonly.web3.eth(),
            readonly.addresses.token,
            include_bytes!("../../artifacts/contracts/Token.sol/Token.json"),
        )?;
    let tx = token.signed_call(
        "bidOn",
        (bid_date.timestamp(), crypto_amount, crypto_account),
        Options::default(),
        readonly.ethereum_key.clone(), // TODO: seems to claim that it's insecure: https://docs.rs/web3/latest/web3/signing/trait.Key.html
    ).await?;

    // FIXME: wait for confirmations before writing to DB
    // let receipt = instance
    //     .my_important_function()
    //     .poll_interval(Duration::from_secs(5))
    //     .confirmations(2)
    //     .execute_confirm()
    //     .await?;

    Ok(tx)
}

#[derive(Deserialize)]
pub struct ConfirmPaymentForm {
    payment_intent_id: String,
    crypto_account: String,
    bid_date: String,
}

async fn fiat_to_crypto(readonly: &Arc<CommonReadonly>, fiat_amount: i64) -> Result<i64, MyError> {
    let price_oracle =
        Contract::from_json(
            readonly.web3.eth(),
            readonly.addresses.collateral_oracle,
            include_bytes!("../../artifacts/@chainlink/contracts/src/v0.7/interfaces/AggregatorV3Interface.sol/AggregatorV3Interface.json"),
        )?;

    // TODO: Query `decimals` only once.
    let accounts = readonly.web3.eth().accounts().await?;
    let decimals = price_oracle.query("decimals", (accounts[0],), None, Options::default(), None).await?;
    let (
        _round_id,
        answer,
        _started_at,
        _updated_at,
        _answered_in_round,
    ): ([u8; 80], [u8; 256], [u8; 256], [u8; 256], [u8; 80]) =
        price_oracle.query("latestRoundData", (accounts[0],), None, Options::default(), None).await?;
    let answer = <u64>::from_le_bytes(answer[..8].try_into().unwrap()) as i64;
    Ok(fiat_amount * i64::pow(10, decimals) / answer) // FIXME: add our "tax"
}

// FIXME: Queue this to the DB for the case of interruption.
// FIXME: Both this and /create-payment-intent only for authenticated and KYC-verified users.
#[post("/confirm-payment")]
pub async fn confirm_payment(
    form: web::Form<ConfirmPaymentForm>,
    ident: Identity,
    common: web::Data<Arc<Mutex<Common>>>,
    readonly: web::Data<CommonReadonly>,
) -> Result<impl Responder, MyError> {
    let client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()?;
    let url = format!("https://api.stripe.com/v1/payment_intents/{}", form.payment_intent_id);
    let intent: Value = client.get(url)
        .basic_auth::<&str, &str>(&readonly.config.stripe.secret_key, None)
        .send().await?
        .json().await?;
    if intent.get("metadata[user]").unwrap().as_str().unwrap() != ident.id()? { // TODO: unwrap()
        return Err(AuthenticationFailedError::new().into());
    }

    if intent.get("currency").unwrap().as_str() != Some("usd") { // TODO: unwrap()
        return Ok(HttpResponse::BadRequest().body("Wrong currency")); // TODO: JSON
    }
    let fiat_amount = intent.get("amount").unwrap().as_i64().unwrap(); // TODO: unwrap()

    match intent.get("status").unwrap().as_str().unwrap() { // TODO: unwrap()
        "succeeded" => {
            use crate::schema::txs::dsl::*;
            let collateral_amount = fiat_to_crypto(&*readonly, fiat_amount).await?;
            // FIXME: Transaction.
            lock_funds(&common, collateral_amount).await?;
            finalize_payment(form.payment_intent_id.as_str(), &*readonly).await?;
            insert_into(txs).values(&(
                // user_id.eq(??), // FIXME
                eth_account.eq(<Address>::from_str(&form.crypto_account)?.as_bytes()),
                usd_amount.eq(fiat_amount),
                crypto_amount.eq(collateral_amount),
                bid_date.eq(DateTime::parse_from_rfc3339(form.bid_date.as_str())?.timestamp()),
            ))
                .execute(&mut common.lock().await.db)?;
            common.lock().await.notify_transaction_tx.send(());
        }
        "canceled" => {
            lock_funds(&common, -fiat_amount).await?;
        }
        _ => {}
    }
    Ok(HttpResponse::Ok().append_header((CONTENT_TYPE, "application/json")).body("{}"))
}

pub async fn exchange_item(item: crate::models::Tx, common: &Arc<Mutex<Common>>, readonly: &Arc<CommonReadonly>) -> Result<(), MyError> {
    lock_funds(common, -item.usd_amount).await?;
    let naive = NaiveDateTime::from_timestamp(item.bid_date, 0);
    let tx = do_exchange(
        &readonly,
        (<&[u8; 20]>::try_from(item.eth_account.as_slice())?).into(),
        DateTime::from_utc(naive, Utc),
        item.crypto_amount
    ).await?;
    use crate::schema::txs::dsl::*;
    update(txs.filter(id.eq(item.id)))
        .set((status.eq(TxsStatusType::SubmittedToBlockchain), tx_id.eq(tx.as_bytes())))
        .execute(&mut common.lock().await.db)?;
    common.lock().await.transactions_awaited.insert(tx);
    Ok(())
}