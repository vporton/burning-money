use std::collections::HashMap;
use actix_web::{Responder, get, HttpResponse, web};
use actix_web::http::header::LOCATION;
// use stripe::{CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession, CreateCheckoutSessionLineItems, CreatePrice, CreateProduct, Currency, IdOrCreate, Price, Product};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::{Common, MyError};

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

#[get("/create-payment-intent")]
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
    // FIXME
    // Ok(HttpResponse::Ok().body(res.text().await?))
    #[derive(Deserialize, Serialize)]
    struct Data {
        id: String,
        client_secret: String,
    }
    let data: Data = serde_json::from_slice(res.bytes().await?.as_ref())?;
    Ok(web::Json(data))
}