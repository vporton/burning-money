use actix_identity::Identity;
use actix_web::{get, post, HttpMessage, HttpRequest, Responder, web, HttpResponse};
use diesel::{ExpressionMethods, insert_into, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use crate::{Common, MyError};
use crate::errors::AuthenticationFailedError;

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

#[derive(Clone, Deserialize)]
pub struct User {
    id: Option<u64>,
    first_name: String,
    last_name: String,
    email: String,
    password: String,
}

#[post("/register")]
pub async fn user_register(request: HttpRequest, info: web::Form<User>, common: web::Data<Common>)
    -> Result<impl Responder, MyError>
{
    // FIXME: Check email.
    let info = info.clone();
    use crate::schema::users::dsl::*;
    let v_id: i64 = insert_into(users).values(
        &(
            first_name.eq(info.first_name),
            last_name.eq(info.last_name),
            email.eq(info.email),
            password.eq(info.password), // FIXME: Check for strong password. // FIXME: Cipher password
        )
    )
        // .execute(&*common.db_pool.get()?)?;
        .returning(id)
        .get_result(&*common.db_pool.get()?)?;
    Identity::login(&request.extensions(), format!("{}", v_id))?;
    Ok(web::Json(""))
}

#[derive(Deserialize)]
pub struct Login {
    email: String,
    password: String,
}

#[post("/login")]
pub async fn user_login(request: HttpRequest, info: web::Form<Login>, common: web::Data<Common>) -> Result<impl Responder, MyError> {
    use crate::schema::users::dsl::*;
    let (v_id, v_password) = users
        .filter(email.eq(info.email.clone()))
        .select((id, password))
        .get_result::<(i64, String)>(&*common.db_pool.get()?)?;
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