use actix_web::{http::header::ContentType, web::{self, ServiceConfig}, HttpResponse, Responder};
use chrono::Utc;
use diesel::{ ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use crate::{database::db::get_connection, http::{requests::user::user_store_request::UserStoreRequest, responses::user::user_store_response::{UserStoreError, UserStoreResponse}}, model::user::{user::User, user_dto::UserDTO}, schema::users};

pub async fn store(body: web::Json<UserStoreRequest>) -> impl Responder {
    let mut data = body.into_inner();
    
    data.password = match bcrypt::hash(&data.password, 10) {
        Ok(password) => password,
        Err(_err) => panic!("Error while bcrypt password")
    };

    let new_user = UserDTO{
        name: data.name, 
        email: data.email,
        password: data.password,
        created_at: Some(Local::now().to_string()), 
        updated_at: Some(String::from("")), 
        deleted_at: Some(String::from(""))
    };

    let token = encode_jwt(new_user);

    HttpResponse::Ok().body(format!("{:#?}", token))
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