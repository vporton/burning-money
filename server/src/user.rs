use std::sync::Arc;
use actix_identity::Identity;
use actix_web::{get, post, HttpMessage, HttpRequest, Responder, web, HttpResponse};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::{Common, MyError};
use crate::errors::AuthenticationFailedError;

#[get("/identity")]
pub async fn user_identity(user: Option<Identity>, common: web::Data<Arc<Mutex<Common>>>) -> Result<impl Responder, MyError> {
    #[derive(Serialize)]
    struct MyIdentity {
        id: Option<i64>,
        kyc: bool,
    }
    let result = if let Some(user) = user {
        let id = user.id()?.parse::<i64>()?;
        let conn = &common.lock().await.db;
        let kyc = conn.query_one("SELECT passed_kyc FROM users WHERE id=$1", &[&id]).await?.get(0);
        MyIdentity {
            id: Some(id),
            kyc,
        }
    } else {
        MyIdentity {
            id: None,
            kyc: false,
        }
    };
    Ok(web::Json(result))
}

#[get("/email")]
pub async fn user_email(user: Option<Identity>, common: web::Data<Arc<Mutex<Common>>>) -> Result<impl Responder, MyError> {
    #[derive(Serialize)]
    struct MyEmail {
        email: Option<String>,
    }
    let result = if let Some(user) = user {
        let email = common.lock().await.db
            .query_one("SELECT email FROM users WHERE id=$1", &[&user.id().unwrap().parse::<i64>()?]).await? // FIXME: query_opt
            .get(0);
        MyEmail {
            email: Some(email),
        }
    } else {
        MyEmail {
            email: None
        }
    };
    Ok(web::Json(result))
}

#[derive(Clone, Deserialize)]
pub struct User {
    // id: Option<u64>,
    first_name: String,
    last_name: String,
    email: String,
    password: String,
}

#[post("/register")]
pub async fn user_register(request: HttpRequest, info: web::Form<User>, common: web::Data<Arc<Mutex<Common>>>)
    -> Result<impl Responder, MyError>
{
    // FIXME: Check email.
    let info = info.clone();
    let common = &**common;
    let v_id: i64 = { // restrict lock duration
        let conn = &common.lock().await.db;
        conn.query_one(
            "INSERT INTO users (first_name, last_name, email, password) VALUES($1, $2, $3, $4) RETURNING id",
            &[&info.first_name, &info.last_name, &info.email, &info.password],
        ).await?.get(0)
    };
    Identity::login(&request.extensions(), format!("{}", v_id))?;
    Ok(web::Json(""))
}

#[derive(Deserialize)]
pub struct Login {
    email: String,
    password: String,
}

#[post("/login")]
pub async fn user_login(request: HttpRequest, info: web::Form<Login>, common: web::Data<Arc<Mutex<Common>>>) -> Result<impl Responder, MyError> {
    let row = { // restrict lock duration
        let conn = &common.lock().await.db;
        conn.query_one(
            "SELECT id, password FROM users WHERE email=$1", &[&info.email]).await?
    };
    let (v_id, v_password): (i64, &str) = (row.get(0), row.get(1));
    if v_password != info.password {
        return Err(AuthenticationFailedError::new().into());
    }
    Identity::login(&request.extensions(), format!("{}", v_id))?;
    Ok(web::Json(""))
}

#[post("/logout")]
pub async fn user_logout(user: Identity) -> impl Responder {
    user.logout();
    HttpResponse::Ok()
}