use actix_web::{http::header::ContentType, web::{self, ServiceConfig}, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use totp_rs::{Algorithm, Secret, TOTP};
use dotenvy_macro::dotenv;
use crate::{database::db::get_connection, http::{requests::auth::auth_login_request::AuthLoginRequest, responses::auth::auth_login_response::{AuthLoginError, AuthLoginResponse}, GenericError, GenericResponse}, model::user::user::User, schema::users, services::auth::{decode_jwt, encode_jwt}};

pub async fn login(body: web::Json<AuthLoginRequest>) -> impl Responder {
    
    let conn = &mut get_connection().await.unwrap();

    // Consulta o usuario a partir do email fornecido no body da requisição
    let get_user = users::table
        .filter(users::email.eq(&body.email))
        .select(User::as_select())
        .get_result::<User>(conn)
        .await;
    
    // Verificar resultado da consulta
    match get_user {
        Ok(user) => {
            
            //Verificar se o usuario possui 2FA ativado
            match user.two_factor_confirmed_at {
                Some(_confirmed_at) => {
                        
                        // Verificar se a senha está correta
                        let result = bcrypt::verify(body.password.as_bytes(),&user.password.as_ref()).unwrap();

                        if result == true {
                            let app_name = dotenv!("APP_NAME");
                            
                            // Aproximar o valor da data/hora para o horario Unix (segundos desde 01/01/1970)
                            let seconds_now = ((Utc::now().timestamp_millis()) / 1000) as u64;
                            
                            let totp = TOTP::new(
                                Algorithm::SHA512,
                                6,
                                1,
                                30,
                                Secret::Encoded(user.two_factor_secret.unwrap()).to_bytes().unwrap(),
                                Some(app_name.to_string()),
                                user.email.clone()
                            ).unwrap();

                            /* 
                                * Pegar a referência do código passado na requisição 
                                * (Se não tiver, pega uma ref. de String vazia)
                            */
                            let code = &body.code.clone().unwrap_or_else(|| "".to_owned());

                            // Checar se o código é valido, dado o horario Unix
                            if totp.check(code, seconds_now) == true {

                                // Se válido, retornar o token de acesso para o usuário
                                let token = encode_jwt(user.email);        
                                
                                let response = AuthLoginResponse {
                                    message: "Login successful",
                                    token: Some(&token)
                                }; 
                
                                return HttpResponse::Ok()
                                    .content_type(ContentType::json())
                                    .json(response);
                            } else {
                                /*
                                    * Se o código do 2FA não é valido, 
                                    * verificar se o código foi passado e de fato é invalido (Cod. 401)
                                    * ou se estava apenas faltando na requisição (Cod. 400)
                                */
                                match code == "" {
                                    true => {
                                        let response = GenericResponse {
                                            message: "Missing 2FA code in request!"
                                        };
        
                                        return HttpResponse::Unauthorized()
                                            .content_type(ContentType::json())
                                            .json(response);
                                    },
                                    false => {
                                        let response = GenericResponse {
                                            message: "2FA challenge failed! Try again."
                                        };
        
                                        return HttpResponse::BadRequest()
                                            .content_type(ContentType::json())
                                            .json(response);
                                    }
                                }
                            }
                        } else {
                            
                            // Caso email ou senha forem invalidos (COM 2FA)
                            let response = AuthLoginError{
                                message: "Error trying to Login!",
                                error: "Invalid email or password."
                            };
            
                            return HttpResponse::NotFound()
                                .content_type(ContentType::json())
                                .json(response);
                        }
                    },
                None => {

                    // Sem 2FA, apenas verificar se a senha está correta
                    let result = bcrypt::verify(body.password.as_bytes(),&user.password).unwrap();
                    
                    if result == true {
                        
                        // Enviar token e msg de sucesso para o usuário
                        let token = encode_jwt(user.email);        
                        let response = AuthLoginResponse{
                            message: "Login successful",
                            token: Some(&token)
                        }; 
        
                        return HttpResponse::Ok()
                            .content_type(ContentType::json())
                            .json(response);
                    } else {
                        
                        // Caso email ou senha forem invalidos (sem 2FA)
                        let response = AuthLoginError{
                            message: "Error trying to Login!",
                            error: "Invalid email or password."
                        };
        
                        return HttpResponse::NotFound()
                            .content_type(ContentType::json())
                            .json(response);
                    }
                }
            }
            
        },
        Err(_error) => {
            let response = AuthLoginError{
                message: "Error trying to login!",
                error: "Internal Server error looking for user in DB"
            };

            return HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .json(response);
        }
    }
    
}

pub async fn validate_token(req: HttpRequest) -> impl Responder {
    let token = req.headers().get("Authorization").unwrap();
    match decode_jwt(token.to_str().expect("Error casting headervalue to &str")) {
        Ok(claim) => {

            let conn = &mut get_connection().await.unwrap();
                
            let find_user = users::table
                .filter(users::email.eq(&claim.sub))
                .select(User::as_select())
                .get_result::<User>(conn)
                .await;

            match find_user {
                Ok(_user) => HttpResponse::Ok().content_type(ContentType::json()).json(claim),
                Err(_error) => {
                    
                    let user_not_found_response = GenericError {
                        message: "No user Logged",
                        error: "Some error raised on server side!"
                    };

                    HttpResponse::Unauthorized()
                        .content_type(ContentType::json())
                        .json(user_not_found_response)
                }
            }
        },
        Err(_error) => {
            let error_response = GenericError {
                message: "No user Logged",
                error: "Some error raised on server side!"
            };

            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .json(error_response);
        }
    }

}

// Endpoints do controller
pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/auth")
        .route("/login", web::post().to(login))
        .route("/validateToken", web::post().to(validate_token))
    );
}