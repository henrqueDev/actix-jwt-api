use actix_web::{http::header::ContentType, web::{self, ServiceConfig}, HttpRequest, HttpResponse, Responder};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

use crate::{database::db::get_connection, http::{requests::auth::auth_login_request::AuthLoginRequest, responses::auth::auth_login_response::{AuthLoginError, AuthLoginResponse}}, model::user::user::User, schema::users, services::auth::{decode_jwt, encode_jwt}};

pub async fn login(body: web::Json<AuthLoginRequest>) -> impl Responder {
    
    let conn = &mut get_connection().await.unwrap();

    let get_user = users::table
        .filter(users::email.eq(&body.email))
        .select(User::as_select())
        .get_result::<User>(conn)
        .await;

    match get_user {
        Ok(user) => {
            let result = bcrypt::verify(body.password.as_bytes(),&user.password).unwrap();
            if result == true {

                let token = encode_jwt(user.email);        
                let response = AuthLoginResponse{
                    message: String::from("Login successful"),
                    token: Some(token)
                }; 

                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .json(response)
            } else {
                let response = AuthLoginError{
                    message: String::from("Invalid email or password")
                };

                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .json(response)
            }
        },
        Err(_error) => {
            let response = AuthLoginError{
                message: String::from("Invalid email or password")
            };

            HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .json(response)
        }
    }
    
}

    


pub async fn validate_token(req: HttpRequest) -> impl Responder {
    let token = req.headers().get("Authorization").unwrap();
    match decode_jwt(token.to_str().expect("Error casting headervalue to &str")) {
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
        .route("/validateToken", web::post().to(validate_token))
    );
}