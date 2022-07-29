use web3::types::*;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use actix_identity::Identity;
use actix_web::{get, post, Responder, web, HttpResponse};
use actix_web::http::header::CONTENT_TYPE;
use chrono::{DateTime, NaiveDateTime, Utc};
use log::{debug, info};
// use stripe::{CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession, CreateCheckoutSessionLineItems, CreatePrice, CreateProduct, Currency, IdOrCreate, Price, Product};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use web3::contract::{Contract, Options};
use tokio::sync::Mutex;
use crate::{Common, CommonReadonly, MyError};
use crate::errors::{AuthenticationFailedError, KYCError, NotEnoughFundsError, StripeError};

// We follow https://stripe.com/docs/payments/finalize-payments-on-the-server

#[derive(Deserialize)]
pub struct CreateStripeCheckout {
    fiat_amount: f64,
}

#[get("/stripe-pubkey")]
pub async fn stripe_public_key(readonly: web::Data<Arc<CommonReadonly>>) -> impl Responder {
    HttpResponse::Ok().body(readonly.config.stripe.public_key.clone())
}

#[post("/create-payment-intent")]
pub async fn create_payment_intent(
    q: web::Query<CreateStripeCheckout>,
    ident: Identity,
    common: web::Data<Arc<Mutex<Common>>>, readonly: web::Data<Arc<CommonReadonly>>,
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
            return Err(KYCError::new().into());
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
    // info!("STRIPE: {}", String::from_utf8_lossy(res.bytes().await?.as_ref()));
    #[derive(Deserialize, Serialize)]
    struct Data {
        id: String,
        client_secret: String,
    }
    let data: Data = serde_json::from_slice(res.bytes().await?.as_ref())?;
    Ok(web::Json(data))
    // Ok(web::Json("{}"))
}

async fn finalize_payment(payment_intent_id: &str, readonly: &Arc<CommonReadonly>) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()?;
    let url = format!("https://api.stripe.com/v1/payment_intents/{}/confirm", payment_intent_id);
    client.post(url)
        .basic_auth::<&str, &str>(&readonly.config.stripe.secret_key, None)
        .send().await?;
    Ok(())
}

pub async fn lock_funds(common: Arc<Mutex<Common>>, amount: i64) -> Result<(), anyhow::Error> {
    let mut common = common.lock().await; // locks for all duration of the function
    const MAX_GAS: i64 = 30_000_000; // TODO: less
    let locked_funds = if common.locked_funds >= 0 { // hack
        common.locked_funds + MAX_GAS
    } else {
        common.locked_funds - MAX_GAS
    };
    if locked_funds + amount >= common.balance {
        debug!("{} funds cannot be locked.", amount);
        return Err(NotEnoughFundsError::new().into());
    }
    common.locked_funds += amount;
    debug!("{} funds locked.", amount);
    Ok(())
}

// It returns the Ethereum transaction (probably, yet not confirmed).
async fn do_exchange(readonly: &Arc<CommonReadonly>, crypto_account: Address, bid_date: DateTime<Utc>, crypto_amount: i64)
    -> Result<H256, anyhow::Error>
{
    let token =
        Contract::from_json(
            readonly.web3.eth(),
            readonly.addresses.token,
            serde_json::from_slice::<Value>(
                include_bytes!("../../artifacts/contracts/Token.sol/Token.json")
            ).unwrap()["abi"].to_string().as_bytes(),
            // r#"[{"type":"function","name":"bidOn","inputs":[{"name":"_day","type":"uint256"},{"name":"_collateralAmount","type":"uint256"},{"name":"_for","type":"address"}],"outputs":[],"stateMutability":"payable"}]"#
            //     .as_bytes()
        )?;
    let tx = token.signed_call(
        "bidOn",
        (U256::from(bid_date.timestamp()), crypto_account),
        Options::with(|opt| {
            opt.value = Some(U256::from(crypto_amount));
            opt.gas = Some(500000.into()); // TODO
        }),
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

async fn fiat_to_crypto(readonly: &Arc<CommonReadonly>, fiat_amount: i64) -> Result<i64, anyhow::Error> {
    let price_oracle =
        Contract::from_json(
            readonly.web3.eth(),
            readonly.addresses.collateral_oracle,
            // serde_json::from_slice::<Value>(
            //     include_bytes!("../../artifacts/@chainlink/contracts/src/v0.7/interfaces/AggregatorV3Interface.sol/AggregatorV3Interface.json")
            // ).unwrap()["abi"].to_string().as_bytes(),
            r#"[{"type":"function","name":"decimals","inputs":[],"outputs":[{"name":"","type":"uint8"}]},{"type":"function","name":"latestRoundData","inputs":[],"outputs":[{"name":"roundId","type":"bytes10"},{"name":"answer","type":"int256"},{"name":"startedAt","type":"uint256"},{"name":"updatedAt","type":"uint256"},{"name":"answeredInRound","type":"bytes10"}]}]"#
                .as_bytes(),
        )?;

    // TODO: Query `decimals` only once.
    let decimals: u8 = price_oracle.query("decimals", (), None, Options::default(), None).await?;
    let (
        _round_id,
        answer,
        _started_at,
        _updated_at,
        _answered_in_round,
    ): ([u8; 10], U256, U256, U256, [u8; 10]) =
        price_oracle.query("latestRoundData", (), None, Options::default(), None).await?;
    let answer = answer.as_u64() as i64;
    let answer = ((answer as f64) * (1.0 - readonly.config.our_tax)) as i64;
    Ok(fiat_amount * i64::pow(10, decimals as u32) / answer)
}

#[post("/confirm-payment")]
pub async fn confirm_payment(
    form: web::Form<ConfirmPaymentForm>,
    ident: Identity,
    common: web::Data<Arc<Mutex<Common>>>,
    readonly: web::Data<Arc<CommonReadonly>>,
) -> Result<impl Responder, MyError> {
    let client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()?;
    let intent_url = format!("https://api.stripe.com/v1/payment_intents/{}", form.payment_intent_id);
    let intent: Value = client.get(intent_url)
        .basic_auth::<&str, &str>(&readonly.config.stripe.secret_key, None)
        .send().await?
        .json().await?;
    if intent.get("metadata").ok_or(StripeError::new())?
        .get("user").ok_or(StripeError::new())?
        .as_str().ok_or(StripeError::new())? != ident.id()?
    {
        return Err(AuthenticationFailedError::new().into());
    }

    if intent.get("currency").ok_or(StripeError::new())?.as_str().ok_or(StripeError::new())? != "usd" {
        return Err(StripeError::new().into());
    }
    let fiat_amount = intent.get("amount").ok_or(StripeError::new())?.as_i64().ok_or(StripeError::new())?;

    let intent_status = intent.get("status").ok_or(StripeError::new())?.as_str().ok_or(StripeError::new())?;
    info!("Payment intent status: {intent_status}");
    let response = match intent_status {
        "requires_confirmation" => { // FIXME: More statuses.
            let collateral_amount = fiat_to_crypto(&*readonly, fiat_amount).await?;
            let common2 = (**common).clone();
            lock_funds(common2.clone(), collateral_amount).await?;
            let id: i64 = { // restrict lock duration
                let conn = &mut common.lock().await.db;
                conn.query_one(
                    "INSERT INTO txs (payment_intent_id, user_id, eth_account, usd_amount, crypto_amount, bid_date) VALUES($1, $2, $3, $4, $5, $6) RETURNING id",
                    &[
                        &form.payment_intent_id,
                        &ident.id()?.parse::<i64>()?,
                        &<Address>::from_str(&form.crypto_account)?.as_bytes(), // TODO: better error message (not error 500)
                        &fiat_amount,
                        &collateral_amount,
                        &DateTime::parse_from_rfc3339(form.bid_date.as_str())?.timestamp(),
                    ],
                ).await?.get(0)
            };
            if let Err(err) = finalize_payment(form.payment_intent_id.as_str(), &*readonly).await {
                info!("Cannot finalize Stripe payment: {}", err);
                lock_funds(common2.clone(), -collateral_amount).await?;
                let conn = &mut common.lock().await.db; // short lock duration
                conn.execute("DELETE FROM txs WHERE id=$1", &[&id]).await?;
                return Err(err.into());
            }
            { // restrict lock duration
                let conn = &mut common.lock().await.db;
                conn.execute("UPDATE txs SET status='ordered' WHERE id=$1", &[&id]).await?;
            }
            common.lock().await.notify_ordered_tx.send(())?;
            json!({
                "requires_action": false,
                "payment_intent_client_secret": intent.get("client_secret").ok_or(StripeError::new())?
            })
        },
        // "succeeded" => {
        //     json!({"success": true})
        // }
        "canceled" | "payment_failed" => {
            let collateral_amount: i64 = common.lock().await.db.query_one(
                "SELECT crypto_amount FROM txs WHERE payment_intent_id=$1",
                &[&form.payment_intent_id])
                .await?
                .get(0);
            lock_funds((**common).clone(), -collateral_amount).await?;
            common.lock().await.db.execute(
                "DELETE FROM txs WHERE payment_intent_id=$1",
                &[&form.payment_intent_id])
                .await?;
            json!({"success": false})
        }
        "processing" => {
            json!({
                "requires_action": false,
                "payment_intent_client_secret": intent.get("client_secret").ok_or(StripeError::new())?
            })
        }
        _ => {
            json!({
                "requires_action": true,
                "payment_intent_client_secret": intent.get("client_secret").ok_or(StripeError::new())?
            })
        }
    };
    Ok(HttpResponse::Ok().append_header((CONTENT_TYPE, "application/json")).body(response.to_string()))
}

pub async fn exchange_item(item: crate::models::Tx, common: Arc<Mutex<Common>>, readonly: &Arc<CommonReadonly>) -> Result<(), anyhow::Error> {
    lock_funds(common.clone(), -item.crypto_amount).await?;
    let naive = NaiveDateTime::from_timestamp(item.bid_date, 0);

    // First submit to blockchain to avoid double submissions.
    let tx = do_exchange(
        &readonly,
        (<&[u8; 20]>::try_from(item.eth_account.as_slice())?).into(),
        DateTime::from_utc(naive, Utc),
        item.crypto_amount,
    ).await?;
    { // restrict lock duration
        let conn = &common.lock().await.db;
        conn.execute(
            "UPDATE txs SET status='submitted_to_blockchain', tx_id=$2 WHERE id=$1",
            &[&item.id, &tx.as_bytes()]
        ).await?;
    }
    common.lock().await.transactions_awaited.insert(tx);
    Ok(())
}
