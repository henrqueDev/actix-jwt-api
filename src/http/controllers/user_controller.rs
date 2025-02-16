use actix_web::{web::{self, ServiceConfig}, HttpResponse, Responder};
use chrono::Local;
use crate::{http::requests::user::user_store_request::UserStoreRequest, model::user::UserDTO, services::auth::encode_jwt};

pub async fn store(body: web::Json<UserStoreRequest>) -> impl Responder {
    let mut data = body.into_inner();
    
    data.password = match bcrypt::hash(&data.password, 12) {
        Ok(password) => password,
        Err(_err) => panic!("Error while bcrypt password")
    };

    let new_user = UserDTO{
        name: data.name, 
        email: data.email,
        password: data.password,
        created_at: Local::now().to_string(), 
        updated_at: String::from(""), 
        deleted_at: String::from("")
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