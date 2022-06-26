use actix_web::{Responder, get, HttpResponse, web};
use actix_web::http::header::{CONTENT_TYPE, LOCATION};
use askama::Template;
use diesel::{ExpressionMethods, insert_into, RunQueryDsl};
use ethers_core::types::H160;
use serde_json::json;
use serde::Deserialize;
use stripe::{CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession, CreateCheckoutSessionLineItems, CreatePrice, CreateProduct, Currency, IdOrCreate, Price, Product};
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
    fiat_amount: f64,
}

#[get("/create-stripe-checkout")]
pub async fn create_stripe_checkout(q: web::Query<CreateStripeCheckout>, common: web::Data<Common>) -> Result<impl Responder, MyError> {
    let client = Client::new(common.config.stripe.secret_key.clone());

    let product = {
        let mut create_product = CreateProduct::new("Mining");
        Product::create(&client, create_product).await?
    };

    let price = {
        let mut create_price = CreatePrice::new(Currency::USD);
        create_price.product = Some(IdOrCreate::Id(&product.id));
        create_price.unit_amount = Some((q.fiat_amount * 100.0) as i64);
        create_price.expand = &["product"];
        Price::create(&client, create_price).await?
    };

    let mut params =
        CreateCheckoutSession::new("http://test.com/cancel", "http://test.com/success"); // FIXME
    // params.customer = Some(customer.id);
    params.mode = Some(CheckoutSessionMode::Payment);
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        price: Some(price.id.to_string()),
        quantity: Some(1), // FIXME
        ..Default::default()
    }]);
    params.expand = &["line_items", "line_items.data.price.product"];

    let session = CheckoutSession::create(&client, params).await?;
    if let Some(url) = session.url {
        Ok(HttpResponse::TemporaryRedirect().append_header((LOCATION, url)).body(""))
    } else {
        Ok(HttpResponse::Ok().body("Stripe didn't return a URL.")) // FIXME
    }
}

#[derive(Deserialize)]
pub struct CreateAccountQuery {
    user_account: String,
}
