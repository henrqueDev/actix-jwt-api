use actix_web::{http::header::ContentType, middleware::from_fn, web::{self, ServiceConfig}, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::services::google_oauth2::refresh_oauth2_google;

use super::middleware::auth_middleware::auth_middleware;

pub mod user_controller;
pub mod auth_controller;
pub mod email_controller;
pub mod product_controller;
use lazy_static::lazy_static;
use dotenvy_macro::dotenv;

lazy_static! {
    static ref GOOGLE_CLIENT_ID: &'static str = dotenv!("ID_CLIENT_GOOGLE");
    static ref GOOGLE_TOKEN_ID: &'static str = dotenv!("KEY_CLIENT_GOOGLE");
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

async fn debug() -> impl Responder {
    let body = refresh_oauth2_google().await;

    return HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(body);
}

/// Endpoints de debug (pura gambiarra eu sei)
pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/tests")
        .route("/debug", web::post().to(debug))
        .wrap(from_fn(auth_middleware))
    );
}