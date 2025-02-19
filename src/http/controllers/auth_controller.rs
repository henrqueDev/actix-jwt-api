use actix_web::{http::header::ContentType, web::{self, ServiceConfig}, HttpResponse, Responder};
use chrono::Local;

use crate::{http::{requests::auth::auth_login_request::AuthLoginRequest, responses::auth_login_response::AuthLoginResponse}, model::user::UserDTO, services::auth::{decode_jwt, encode_jwt}};

pub async fn login(body: web::Json<AuthLoginRequest>) -> impl Responder {
    
    let hash = match bcrypt::hash(body.password.as_bytes(), 12) {
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
            created_at: Some(Local::now().to_string()), 
            updated_at: Some(String::from("")), 
            deleted_at: Some(String::from(""))
        };

        let token = encode_jwt(some_user_example);        
        let response = AuthLoginResponse{
            message: String::from("Login realizado com sucesso!"),
            token: Some(token)
        }; 

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(response)
    } else {
        let response = AuthLoginResponse{
            message: String::from("Credenciais inválidas, tente novamente!"),
            token: None
        };

        HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .json(response)
    }

}

pub async fn validate_token(body: web::Json<AuthLoginResponse>) -> impl Responder {
    let token = body.token.as_ref().unwrap();
    
    match decode_jwt(&token) {
        Ok(claim) => {
            return HttpResponse::Ok()
                .content_type(ContentType::json())
                .json(claim);
        },
        Err(error) => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .json(error.to_string());
        }
    };

}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/auth")
        .route("/login", web::post().to(login))
        .route("/validate-token", web::post().to(validate_token))
    );
}