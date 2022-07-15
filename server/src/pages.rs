use actix_identity::Identity;
use actix_web::{Responder, get, HttpResponse, web};
use askama::Template;
use diesel::{ExpressionMethods, insert_into, RunQueryDsl};
use ethers_core::types::H160;
use serde_json::json;
use serde::{Deserialize, Serialize};
use crate::Config;
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

#[get("/identity")]
pub async fn user_identity(user: Option<Identity>) -> impl Responder {
    #[derive(Serialize)]
    struct MyIdentity {
        id: Option<String>,
    }
    let result = if let Some(user) = user {
        MyIdentity {
            id: Some(user.id().unwrap()),
        }
    } else {
        MyIdentity {
            id: None
        }
    };
    web::Json(result)
}