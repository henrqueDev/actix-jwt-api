use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;
use dotenvy_macro::dotenv;

use crate::services::redis_client::cache_set_key;

lazy_static! {
    static ref GOOGLE_CLIENT_ID: &'static str = dotenv!("ID_CLIENT_GOOGLE");
    static ref GOOGLE_TOKEN_ID: &'static str = dotenv!("SECRET_CLIENT_GOOGLE");
    static ref REFRESH_TOKEN_GOOGLE: &'static str = dotenv!("REFRESH_TOKEN_GOOGLE");
    static ref REDIRECT_URI_GOOGLE: &'static str = dotenv!("REDIRECT_URI_GOOGLE");
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
pub struct RefreshTokenResponse {
    access_token: String,
    expires_in: u64,
    scope: String,
    token_type: String,
    refresh_token_expires_in: u64
}

pub async fn refresh_oauth2_google() -> String {
    let client = reqwest::Client::builder()
        .build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let data = format!("{{
        \"client_id\": \"{}\",
        \"client_secret\": \"{}\",
        \"refresh_token\": \"{}\",
        \"redirect_uri\": \"{}\",
        \"grant_type\": \"refresh_token\"
    }}", GOOGLE_CLIENT_ID.to_owned(), GOOGLE_TOKEN_ID.to_owned(), REFRESH_TOKEN_GOOGLE.to_owned(), REDIRECT_URI_GOOGLE.to_owned());

    let json = serde_json::from_str::<RefreshTokenRequest>(&data).unwrap();

    let request = client.request(reqwest::Method::POST, "https://oauth2.googleapis.com/token")
        .headers(headers)
        .json(&json);

    let response = request.send().await.unwrap();
    let body = serde_json::from_str::<RefreshTokenResponse>(&response.text().await.unwrap()).unwrap();
    
    let _ = cache_set_key::<&str, &String, String>("GOOGLE_OAUTH2_KEY", &body.access_token, 3598).await;

    return body.access_token;
}

