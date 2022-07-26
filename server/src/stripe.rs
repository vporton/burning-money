use web3::types::*;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use actix_identity::Identity;
use actix_web::{get, post, Responder, web, HttpResponse};
use actix_web::http::header::CONTENT_TYPE;
use chrono::{DateTime, NaiveDateTime, Utc};
// use stripe::{CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession, CreateCheckoutSessionLineItems, CreatePrice, CreateProduct, Currency, IdOrCreate, Price, Product};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web3::contract::{Contract, Options};
use tokio::sync::Mutex;
use crate::{Common, CommonReadonly, MyError};
use crate::errors::{AuthenticationFailedError, NotEnoughFundsError, StripeError};

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
        let common = (**common).clone();
        let v_id = ident.id()?.parse::<i64>()?;
        let v_passed_kyc: bool = {
            let result = common.lock().await.db
                .query_one("SELECT passed_kyc FROM users WHERE id=$1", &[&v_id]).await?;
            result.get(0)
        };
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

async fn lock_funds(common: Arc<Mutex<Common>>, amount: i64) -> Result<(), MyError> {
    let mut common = common.lock().await; // locks for all duration of the function
    if common.locked_funds + amount >= common.balance {
        return Err(NotEnoughFundsError::new().into());
    }
    common.locked_funds += amount;
    // let amount = amount.clone();
    // let amount2 = amount.clone(); // superfluous?
    // let mut common = common.lock().await; // locks for all duration of the function
    // let conn = &mut common.db; // locks for all duration of the function
    // let trans = conn.transaction().await?;
    // let result = { // block to limit scope of trans0
    //     let trans0 = &trans;
    //     let do_it = || async move {
    //         let v_free_funds: i64 =
    //             trans0.query_one("SELECT free_funds FROM global FOR UPDATE", &[]).await?.get(0);
    //         const MAX_GAS: i64 = 30_000_000; // TODO: less
    //         if amount >= v_free_funds + MAX_GAS {
    //             return Err::<_, MyError>(NotEnoughFundsError::new().into());
    //         }
    //         trans0.execute("UPDATE global SET free_funds=$1", &[&(v_free_funds - amount2)]).await?;
    //         Ok(())
    //     };
    //     do_it().await
    // };
    // // let trans2 = &mut trans;
    // finish_transaction::<_, MyError>(trans, result).await?;
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
        readonly.ethereum_key.clone(),
    ).await?;

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
    Ok(fiat_amount * i64::pow(10, decimals) / answer)
}

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
    if intent.get("metadata[user]").ok_or(StripeError::new())?.as_str().ok_or(StripeError::new())? != ident.id()? {
        return Err(AuthenticationFailedError::new().into());
    }

    if intent.get("currency").ok_or(StripeError::new())?.as_str().ok_or(StripeError::new())? != "usd" {
        return Err(StripeError::new().into());
    }
    let fiat_amount = intent.get("amount").ok_or(StripeError::new())?.as_i64().ok_or(StripeError::new())?;

    match intent.get("status").ok_or(StripeError::new())?.as_str().ok_or(StripeError::new())? {
        "succeeded" => {
            let collateral_amount = fiat_to_crypto(&*readonly, fiat_amount).await?;
            let common2 = (**common).clone();
            lock_funds(common2.clone(), collateral_amount).await?;
            let id: i64 = { // restrict lock duration
                let conn = &mut common.lock().await.db;
                conn.query_one(
                    "INSERT INTO txs SET user_id=$1, eth_account=$2, usd_amount=$3, crypto_amount=$4, bid_date=$5",
                    &[
                        &ident.id()?.parse::<i64>()?,
                        &<Address>::from_str(&form.crypto_account)?.as_bytes(),
                        &fiat_amount,
                        &collateral_amount,
                        &DateTime::parse_from_rfc3339(form.bid_date.as_str())?.timestamp(),
                    ],
                ).await?.get(0)
            };
            if let Err(err) = finalize_payment(form.payment_intent_id.as_str(), &*readonly).await {
                lock_funds(common2.clone(), -collateral_amount).await?;
                let conn = &mut common.lock().await.db; // short lock duration
                conn.execute("DELETE FROM txs WHERE id=$1", &[&id]).await?;
                return Err(err.into());
            }
            { // restrict lock duration
                let conn = &mut common.lock().await.db;
                conn.execute("UPDATE txs WHERE id=$1 SET status='ordered'", &[&id]).await?;
            }
            common.lock().await.notify_transaction_tx.send(())?;
        }
        "canceled" => {
            lock_funds((**common).clone(), -fiat_amount).await?;
        }
        _ => {}
    }
    Ok(HttpResponse::Ok().append_header((CONTENT_TYPE, "application/json")).body("{}"))
}

pub async fn exchange_item(item: crate::models::Tx, common: Arc<Mutex<Common>>, readonly: &Arc<CommonReadonly>) -> Result<(), MyError> {
    lock_funds(common.clone(), -item.usd_amount).await?;
    let naive = NaiveDateTime::from_timestamp(item.bid_date, 0);
    let tx = do_exchange(
        &readonly,
        (<&[u8; 20]>::try_from(item.eth_account.as_slice())?).into(),
        DateTime::from_utc(naive, Utc),
        item.crypto_amount
    ).await?;
    let conn = &common.lock().await.db;
    conn.execute("UPDATE txs SET status='submitted_to_blockchain' WHERE id=$1", &[&item.id]).await?;
    common.lock().await.transactions_awaited.insert(tx);
    Ok(())
}