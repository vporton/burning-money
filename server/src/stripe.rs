use actix_web::{Responder, get, HttpResponse, web};
use actix_web::http::header::LOCATION;
use stripe::{CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession, CreateCheckoutSessionLineItems, CreatePrice, CreateProduct, Currency, IdOrCreate, Price, Product};
use serde::Deserialize;
use crate::{Common, MyError};

#[derive(Deserialize)]
struct CreateStripeCheckout {
    fiat_amount: f64,
}

#[get("/create-stripe-checkout")]
pub async fn create_stripe_checkout(q: web::Query<CreateStripeCheckout>, common: web::Data<Common>) -> Result<impl Responder, MyError> {
    let client = Client::new(common.config.stripe.secret_key.clone());

    let product = {
        let mut create_product = CreateProduct::new("Mining World Token");
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
