use actix_web::{web::{self, ServiceConfig}, HttpResponse, Responder};
use chrono::Local;

use crate::{http::requests::auth::auth_login_request::AuthLoginRequest, model::user::UserDTO, services::auth::encode_jwt};

pub async fn login(body: web::Json<AuthLoginRequest>) -> impl Responder {
    let data = body.into_inner();
    
    let hash = match bcrypt::hash(data.password.as_bytes(), 12) {
        Ok(hash) => hash,
        Err(_err) => panic!("Some error raised while hashing password")
    };

    let validate = match bcrypt::verify("password".as_bytes(), &hash) {
        Ok(result) => result,
        Err(_err) => panic!("Some error raised while verifying password")
    };

    if validate == true {
        let some_user_example = UserDTO {
            name: String::from("João Embaixadinha"), 
            email: String::from("jaoembaixadinha@gmail.com"),
            password: hash,
            created_at: Local::now().to_string(), 
            updated_at: String::from(""), 
            deleted_at: String::from("")
        };
        let token = encode_jwt(some_user_example);
        HttpResponse::Ok().body(format!("{:#?}", token))
    } else {
        HttpResponse::Unauthorized().body("Credenciais inválidas, tente novamente!")
    }

}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/auth")
        .route("/login", web::post().to(login))
    );
}