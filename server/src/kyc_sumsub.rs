use std::sync::Arc;
use actix_web::{HttpRequest, HttpResponse, post, Responder, web};
use std::time::{SystemTime, UNIX_EPOCH};
use actix_identity::Identity;
use hex::FromHex;
use hmac::{Hmac, Mac};
use reqwest::{Client, Method, Request, Url};
use reqwest::header::HeaderValue;
use sha2::Sha256;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;
use hmacsha1::hmac_sha1;
use crate::{Common, CommonReadonly, MyError};
use crate::errors::{AuthenticationFailedError, MyErrorBase};

fn signature(readonly: &CommonReadonly, timestamp: u64, http_method: &str, path: &str, body: &[u8]) -> String {
    let catenated = [format!("{}", timestamp).as_bytes(), http_method.as_bytes(), path.as_bytes(), body].concat();

    let mut mac = Hmac::<Sha256>::new_from_slice(readonly.config.secrets.sumsub_secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(catenated.as_slice());
    let sig = mac.finalize();
    hex::encode(sig.into_bytes()) // lowercase
}

fn modify_request(readonly: &Arc<CommonReadonly>, req: &mut Request) -> Result<(), anyhow::Error> {
    let timestamp: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let full_path = if let Some(query) = req.url().query() {
        req.url().path().to_string() + "?" + query
    } else {
        req.url().path().to_string()
    };
    let body = if let Some(body) = req.body() {
        if let Some(bytes) = body.as_bytes() {
            bytes
        } else {
            panic!("Body cannot be a stream.");
        }
    } else {
        &[]
    };
    let sig = signature(readonly, timestamp, req.method().as_str(), full_path.as_str(), body);
    req.headers_mut().insert("X-App-Token", HeaderValue::from_str(readonly.config.secrets.sumsub_access_token.as_str())?);
    req.headers_mut().insert("X-App-Access-Sig", HeaderValue::from_str(sig.as_str())?);
    req.headers_mut().insert("X-App-Access-Ts", HeaderValue::from_str(timestamp.to_string().as_str())?);

    Ok(())
}

#[post("/kyc/access-token")]
pub async fn sumsub_generate_access_token(
    ident: Identity,
    common: web::Data<Arc<Mutex<Common>>>,
    readonly: web::Data<Arc<CommonReadonly>>
) -> Result<String, MyError>
{
    let conn = &common.lock().await.db;
    let id = ident.id()?.parse::<i64>()?;
    let email: String = conn.query_one("SELECT email FROM users WHERE id=$1", &[&id]).await?.get(0);
    let client = Client::new();
    let url = format!("https://api.sumsub.com/resources/applicants?levelName=basic-kyc-level");
    let body = json!({"externalUserId": id.to_string(), "email": email});
    let mut request = Request::new(Method::POST, Url::parse(url.as_str())?);
    *request.body_mut() = Some(body.to_string().into());
    modify_request(&*readonly, &mut request)?;
    let _res = client.execute(request).await?; // TODO: What they return on first and on second request?
    let url = format!(
        "https://api.sumsub.com/resources/accessTokens?userId={}&levelName=basic-kyc-level",
        id,
    );
    let mut request = Request::new(Method::POST, Url::parse(url.as_str())?);
    modify_request(&*readonly, &mut request)?;
    #[derive(Deserialize)]
    struct TokenResponse {
        token: String,
    }
    let res = client.execute(request).await?.json::<TokenResponse>().await?;
    Ok(res.token.to_string())
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct ReviewResult {
    reviewAnswer: String,
    // moderationComment: String, // to send a email to the user
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct ReviewedHook {
    applicantId: String,
    reviewResult: ReviewResult,
}

#[post("/kyc/reviewed_hook")]
pub async fn sumsub_reviewed_hook(
    body: web::Bytes,
    req: HttpRequest,
    common: web::Data<Arc<Mutex<Common>>>,
    readonly: web::Data<Arc<CommonReadonly>>,
) -> impl Responder {
    if let Err(err) = do_sumsub_reviewed_hook(body, req, common, readonly).await {
        match err {
            MyErrorBase::AuthenticationFailed(_) | MyErrorBase::ActixHeaderToStr(_) =>
                HttpResponse::BadRequest().body(""),
            _ =>
                HttpResponse::InternalServerError().body(""),
        }
    } else {
        HttpResponse::Ok().body("")
    }
}

// TODO: Send email (or SumSub does it itself?)
async fn do_sumsub_reviewed_hook(
    body: web::Bytes,
    req: HttpRequest,
    common: web::Data<Arc<Mutex<Common>>>,
    readonly: web::Data<Arc<CommonReadonly>>,
) -> Result<(), MyErrorBase> {
    let digest = req.headers().get("x-payload-digest").ok_or(AuthenticationFailedError::new())?.to_str()?;
    let digest = <[u8; 20]>::from_hex(digest)?;
    let body = &*body;
    let hmac = hmac_sha1(readonly.config.secrets.sumsub_secret_key.as_bytes(), body);
    if hmac != digest {
        return Err(AuthenticationFailedError::new().into());
    }
    let j: ReviewedHook = serde_json::from_slice(body)?;
    if j.reviewResult.reviewAnswer == "GREEN" {
        let conn = &common.lock().await.db;
        let client_id = j.applicantId.parse::<i64>()?;
        conn.execute("UPDATE users SET kyc=TRUE WHERE id=$1", &[&client_id]).await?;
    }
    Ok(())
}