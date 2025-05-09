use actix_web::{http::header::ContentType, web::{self, ServiceConfig}, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;
use dotenvy_macro::dotenv;

use crate::{http::GenericError, services::redis_client::cache_set_key};

use super::{brute_force_protection::brute_force_protection, redis_client::cache_get_key};

lazy_static! {
    static ref GOOGLE_CLIENT_ID: &'static str = dotenv!("ID_CLIENT_GOOGLE");
    static ref GOOGLE_TOKEN_ID: &'static str = dotenv!("SECRET_CLIENT_GOOGLE");
    static ref REDIRECT_URI_GOOGLE: &'static str = dotenv!("REDIRECT_URI_GOOGLE");
}

#[derive(Deserialize)]
pub struct SetRefreshTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String,
    pub token_type: String,
    pub refresh_token_expires_in: u64
}

#[derive(Deserialize)]
pub struct SetRefreshTokenRequest {
    pub state: String,
    pub code: String,
    pub scope: String
}

#[derive(Deserialize, Serialize)]
struct RefreshTokenRequest {
    client_id: String,
    client_secret: String,
    refresh_token: String,
    redirect_uri: String,
    grant_type: String
}

#[derive(Deserialize, Serialize)]
struct UpdateOAuth2RefreshTokenRequest {
    code: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    grant_type: String
}

#[derive(Deserialize, Serialize)]
pub struct RefreshTokenResponse {
    access_token: String,
    expires_in: u64,
    scope: String,
    token_type: String,
    refresh_token_expires_in: u64
}

pub async fn refresh_oauth2_google() -> Result<String, u16> {
    let client = reqwest::Client::builder()
        .build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let refresh_token = match cache_get_key::<&str, String>("GOOGLE_OAUTH2_REFRESH_TOKEN").await {
        Ok(refresh_token_string) => refresh_token_string,
        Err(_) => {
            return Err(500);
        }
    };

    let data = format!("{{
        \"client_id\": \"{}\",
        \"client_secret\": \"{}\",
        \"refresh_token\": \"{}\",
        \"redirect_uri\": \"{}\",
        \"grant_type\": \"refresh_token\"
    }}", GOOGLE_CLIENT_ID.to_owned(), GOOGLE_TOKEN_ID.to_owned(), refresh_token, REDIRECT_URI_GOOGLE.to_owned());

    let json = serde_json::from_str::<RefreshTokenRequest>(&data).unwrap();

    let request = client.request(reqwest::Method::POST, "https://oauth2.googleapis.com/token")
        .headers(headers)
        .json(&json);

    let response = request.send().await.unwrap();
    let body = serde_json::from_str::<RefreshTokenResponse>(&response.text().await.unwrap());

    match body {
        Ok(res) => {
            let _ = cache_set_key::<&str, &String, String>("GOOGLE_OAUTH2_KEY", &res.access_token, 3598).await;

            return Ok(res.access_token);
        },
        Err(_) => {
            return Err(500);
        }
    }
}

async fn set_refresh_token_config(req: HttpRequest, query: web::Query<SetRefreshTokenRequest>) -> impl Responder {
    let client = reqwest::Client::builder()
    .build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let params = query.0;

    let data = format!("{{
        \"code\": \"{}\",
        \"client_id\": \"{}\",
        \"client_secret\": \"{}\",
        \"redirect_uri\": \"{}\",
        \"grant_type\": \"authorization_code\"
    }}", params.code, GOOGLE_CLIENT_ID.to_owned(), GOOGLE_TOKEN_ID.to_owned(), REDIRECT_URI_GOOGLE.to_owned());

    let json = serde_json::from_str::<UpdateOAuth2RefreshTokenRequest>(&data).unwrap();

    let request = client.request(reqwest::Method::POST, "https://oauth2.googleapis.com/token")
        .headers(headers)
        .json(&json);

    let response = request.send().await.unwrap();
    let response_status = response.status().as_u16();

    if response_status == 200 {
        let body = serde_json::from_str::<SetRefreshTokenResponse>(&response.text().await.unwrap()).unwrap();
        let _ = cache_set_key::<&str, &String, String>("GOOGLE_OAUTH2_REFRESH_TOKEN", &body.refresh_token, 60000).await;

        return HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body("OAuth2 Google token updated! You can close this window and return to the app.");
    } else if response_status >= 400 && response_status < 500 {
        let res_err = GenericError {
            message: "Error trying refreshing OAuth2 Google token!",
            error: "Internal server error raised"
        };

        brute_force_protection(req).await;

        return HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .json(res_err);
    } else {
        let res_err = GenericError {
            message: "Error trying refreshing OAuth2 Google token!",
            error: "Internal server error raised"
        };

        return HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .json(res_err);
    }
}

/// Endpoints de configs
pub fn config(cfg: &mut ServiceConfig) -> () {
    cfg.service(
web::scope("/configs")
            .route("/set_oauth2_code", web::get().to(set_refresh_token_config))
    );
}