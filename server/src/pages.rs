use actix_web::{Responder, get, HttpResponse, web};
use actix_web::http::header::{CONTENT_TYPE, LOCATION};
use askama::Template;
use diesel::{ExpressionMethods, insert_into, RunQueryDsl};
use ethers_core::types::H160;
use serde_json::json;
use serde::Deserialize;
use stripe::{CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession, CreateCheckoutSessionLineItems};
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
struct CreateStripeCheckout {
    price: f64,
    quantity: f64,
}

#[get("/create-stripe-checkout")]
pub async fn create_stripe_checkout(q: web::Query<CreateStripeCheckout>, config: web::Data<Config>) -> Result<impl Responder, MyError> {
    let client = Client::new(config.stripe.secret_key.clone());

    let mut params =
        CreateCheckoutSession::new("http://test.com/cancel", "http://test.com/success");
    // params.customer = Some(customer.id);
    params.mode = Some(CheckoutSessionMode::Payment);
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        quantity: Some((q.quantity * 1e18) as u64), // FIXME
        price: Some((q.price / 1e18).to_string()), // FIXME
        // amount: Some((q.quantity * 1e18) as u64), // FIXME
        ..Default::default()
    }]);
    params.expand = &["line_items", "line_items.data.price.product"];

    let session = CheckoutSession::create(&client, params).await.unwrap(); // FIXME: unwrap();
    Ok(HttpResponse::TemporaryRedirect().append_header((LOCATION, session.url.unwrap())).body("")) // FIXME: unwrap();
}

#[derive(Deserialize)]
pub struct CreateAccountQuery {
    user_account: String,
}
