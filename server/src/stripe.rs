use ethcontract::transaction::Account;
use std::time::{Duration, SystemTime};
use web3::api::Web3;
use web3::types::*;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use actix_web::{Responder, get, post, HttpResponse, web};
use actix_web::http::header::LOCATION;
use chrono::{DateTime, Utc};
use ethkey::{EthAccount};
// use stripe::{CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession, CreateCheckoutSessionLineItems, CreatePrice, CreateProduct, Currency, IdOrCreate, Price, Product};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use toml::value::Datetime;
use web3::contract::{Contract, Options};
use web3::transports::Http;
use crate::{Common, MyError, stripe};

// We follow https://stripe.com/docs/payments/finalize-payments-on-the-server

#[derive(Deserialize)]
pub struct CreateStripeCheckout {
    fiat_amount: f64,
}

// #[get("/create-stripe-checkout")]
// pub async fn create_stripe_checkout(q: web::Query<CreateStripeCheckout>, common: web::Data<Common>) -> Result<impl Responder, MyError> {
//     let client = Client::new(common.config.stripe.secret_key.clone());
//
//     let product = {
//         let create_product = CreateProduct::new("Mining CardToken");
//         Product::create(&client, create_product).await?
//     };
//
//     let price = {
//         let mut create_price = CreatePrice::new(Currency::USD);
//         create_price.product = Some(IdOrCreate::Id(&product.id));
//         create_price.unit_amount = Some((q.fiat_amount * 100.0) as i64);
//         create_price.expand = &["product"];
//         Price::create(&client, create_price).await?
//     };
//
//     let mut params =
//         CreateCheckoutSession::new("http://test.com/cancel", "http://test.com/success"); // FIXME
//     // params.customer = Some(customer.id);
//     params.mode = Some(CheckoutSessionMode::Payment);
//     params.line_items = Some(vec![CreateCheckoutSessionLineItems {
//         price: Some(price.id.to_string()),
//         quantity: Some(1), // FIXME
//         ..Default::default()
//     }]);
//     params.expand = &["line_items", "line_items.data.price.product"];
//
//     let session = CheckoutSession::create(&client, params).await?;
//     if let Some(url) = session.url {
//         Ok(HttpResponse::TemporaryRedirect().append_header((LOCATION, url)).body(""))
//     } else {
//         Ok(HttpResponse::Ok().body("Stripe didn't return a URL.")) // FIXME
//     }
// }

#[get("/stripe-pubkey")]
pub async fn stripe_public_key(common: web::Data<Common>) -> impl Responder {
    HttpResponse::Ok().body(common.config.stripe.public_key.clone())
}

#[post("/create-payment-intent")]
pub async fn create_payment_intent(q: web::Query<CreateStripeCheckout>, common: web::Data<Common>) -> Result<impl Responder, MyError> {
    let client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()?;
    let mut params = HashMap::new();
    let fiat_amount = q.fiat_amount.to_string();
    params.insert("amount", fiat_amount.as_str());
    params.insert("currency", "usd");
    params.insert("automatic_payment_methods[enabled]", "true");
    params.insert("secret_key_confirmation", "required");
    let res = client.post("https://api.stripe.com/v1/payment_intents")
        .basic_auth::<&str, &str>(&common.config.stripe.secret_key, None)
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

async fn finalize_payment(payment_intent_id: &str, common: &Common) -> Result<(), MyError> {
    let client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()?;
    let url = format!("https://api.stripe.com/v1/payment_intents/{}/confirm", payment_intent_id);
    client.post(url)
        .basic_auth::<&str, &str>(&common.config.stripe.secret_key, None)
        .send().await?;
    Ok(())
}

fn lock_funds(amount: i64) -> Result<(), MyError> {
    // FIXME
}

ethcontract::contract!("../artifacts/@chainlink/contracts/src/v0.7/interfaces/AggregatorV3Interface.sol/AggregatorV3Interface.json");

async fn do_exchange(web3: &Web3<Http>, addresses: &Value, common: Common, crypto_account: H160, bid_date: String, crypto_amount: i64) -> Result<(), MyError> {
    let bid_date: DateTime<Utc> = bid_date.parse()?;

    let token =
        Contract::from_json(
            web3.eth(),
            <H160>::from_str(&addresses["Token"].to_string())?,
            include_bytes!("../../artifacts/contracts/Token.sol/Token.json"),
        )?;
    let tx = token.signed_call(
        "bidOn",
        (bid_date.timestamp(), crypto_amount, crypto_account),
        Options::default(),
        common.ethereum_key.into(),
    ).await?;

    // FIXME: wait for confirmations before writing to DB
    // let receipt = instance
    //     .my_important_function()
    //     .poll_interval(Duration::from_secs(5))
    //     .confirmations(2)
    //     .execute_confirm()
    //     .await?;

    Ok(())
}

#[derive(Deserialize)]
struct ConfirmPaymentForm {
    payment_intent_id: String,
    crypto_account: String,
    bid_date: String,
}

async fn fiat_to_crypto(web3: &Web3<Http>, addresses: &Value, fiat_amount: i64) -> Result<i64, MyError> {
    // TODO: Refactor below to a separate function:
    let price_oracle = AggregatorV3Interface::at(web3, addresses.get("oracle_address")?).await?;

    // TODO: Query `decimals` only once.
    let decimals = price_oracle
        .decimals()
        // .from(ethereum_key)
        .execute()
        .await?;
    let (
        roundId,
        answer,
        startedAt,
        updatedAt,
        answeredInRound
    ) = price_oracle
        .latestRoundData()
        // .from(ethereum_key)
        .execute()
        .await?;

    Ok(fiat_amount * 10**(decimals + 2) / fiat_amount) // FIXME: add our "tax"
}

// FIXME: Queue this to the DB for the case of interruption.
#[post("/confirm-payment")]
pub async fn confirm_payment(form: web::Form<ConfirmPaymentForm>, common: web::Data<Common>) -> Result<impl Responder, MyError> {
    // let stripe_client = stripe::Client::new(&common.config.stripe.secret_key);

    let client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()?;
    let url = format!("https://api.stripe.com/v1/payment_intents/{}", form.payment_intent_id);
    let intent = client.get(url)
        .basic_auth::<&str, &str>(&common.config.stripe.secret_key, None)
        .send().await?
        .json().await?;

    if intent.get("currency") != Some("usd") {
        return Ok(HttpResponse::BadRequest().body("Wrong currency")); // TODO: JSON
    }
    let fiat_amount = intent.get("amount")?.parse::<i64>()?;

    if intent.get("status") == Some("succeeded") {
        let transport = Http::new(&common.config.ethereum_endpoint)?;
        let web3 = Web3::new(transport)?;

        let addresses: Value = serde_json::from_str(fs::read_to_string(common.config.addresses_file)?); // TODO: Don't read and parse it each time. // TODO: more specific type
        let addresses = addresses.get(common.config.ethereum_network)?;

        let collateral_amount = fiat_to_crypto(&web3, addresses, fiat_amount).await?;
        lock_funds(collateral_amount)?;
        let result = finalize_payment(form.payment_intent_id.as_str(), common.get_ref()).await;
        do_exchange(&web3, addresses, (*common).get_ref(), <H160>::from_str(&form.crypto_account)?, form.bid_date.parse_from_rfc3339(), collateral_amount).await?;
        lock_funds(-collateral_amount)?;
        result?;
    } else {
        // TODO
    }
    Ok(web::Json(json!({})))
}