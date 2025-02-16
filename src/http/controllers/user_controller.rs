use actix_web::{web::{self, ServiceConfig}, HttpResponse, Responder};
use crate::http::requests::user::user_store_request::UserStoreRequest;

pub async fn store(body: web::Json<UserStoreRequest>) -> impl Responder {
    HttpResponse::Ok().body(format!("{:#?}", body))
}

pub async fn users() -> impl Responder {
    HttpResponse::Ok().body("Bem vindo ao PetHotel API! Rota: Users")
}

pub async fn other() -> impl Responder {
    HttpResponse::Ok().body("Bem vindo ao PetHotel API! Rota: Users")
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
web::scope("/users")
            .route("/", web::get().to(users))
            .route("/other", web::get().to(other))
            .route("/store", web::post().to(store))
    );
}