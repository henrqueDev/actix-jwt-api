use actix_web::{http::header::ContentType, middleware::from_fn, web::{self, ServiceConfig}, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use totp_rs::{Algorithm, Secret, TOTP};
use dotenvy_macro::dotenv;
use validator::Validate;
use crate::{database::db::get_connection, http::{middleware::auth_middleware::auth_middleware, requests::auth::auth_login_request::AuthLoginRequest, responses::auth::auth_login_response::{AuthLoginError, AuthLoginResponse}, GenericError, GenericResponse}, models::user::user::User, schema::users, services::{auth::{decode_jwt, encode_jwt}, brute_force_protection::{brute_force_protection, remove_brute_force_protection}}};

/// Endpoint para o usuário efetuar o Login
pub async fn login(req: HttpRequest, body: web::Json<AuthLoginRequest>) -> impl Responder {
    let validate = body.validate();
    
    match validate {
        Ok(_) => {
            let conn = &mut get_connection().await.unwrap();

            // Consulta o usuario a partir do email fornecido no body da requisição
            let get_user = users::table
                .filter(users::email.eq(&body.email.clone().unwrap()))
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
                            match &user.password {
                                Some(user_password) => {
                                    let result = bcrypt::verify(body.password.clone().unwrap().as_bytes(),user_password).unwrap();

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

                                            remove_brute_force_protection(req).await;
                            
                                            return HttpResponse::Ok()
                                                .content_type(ContentType::json())
                                                .json(response);
                                        } else {
                                            /*
                                                * Se o código do 2FA não é valido, 
                                                * verificar se o código foi passado é de fato invalido (Cod. 401)
                                            */
                                            
                                            let response = GenericResponse {
                                                message: "2FA challenge failed! Try again."
                                            };
    
                                            brute_force_protection(req, Some(2)).await;
            
                                            return HttpResponse::Unauthorized()
                                                .content_type(ContentType::json())
                                                .json(response);
                                                
                                        }
                                    } else {
                                        
                                        // Caso email ou senha forem invalidos (COM 2FA)
                                        let response = AuthLoginError{
                                            message: "Error trying to Login!",
                                            error: "Invalid email or password."
                                        };
    
                                        brute_force_protection(req, None).await;
                        
                                        return HttpResponse::NotFound()
                                            .content_type(ContentType::json())
                                            .json(response);
                                    }
                                },
                                None => {
                                    let res_err = AuthLoginError {
                                        message: "You have not set your user password yet!",
                                        error: "Error trying update user!"
                                    };
        
                                    return HttpResponse::Conflict()
                                    .content_type(ContentType::json())
                                    .json(res_err);
                                }
                            }
                            
                        },
                        None => {

                            // Sem 2FA, apenas verificar se a senha está correta
                            match &user.password {
                                Some(user_password) => {
                                    let result = bcrypt::verify(body.password.clone().unwrap().as_bytes(),user_password).unwrap();
                                    
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

                                        brute_force_protection(req, None).await;
                        
                                        return HttpResponse::NotFound()
                                            .content_type(ContentType::json())
                                            .json(response);
                                    }
                                },
                                None => {
                                    let res_err = AuthLoginError {
                                        message: "You have not set your user password yet!",
                                        error: "Error trying to login"
                                    };
        
                                    return HttpResponse::Conflict()
                                    .content_type(ContentType::json())
                                    .json(res_err);
                                }
                            }
                        }
                    }
                    
                },
                Err(_error) => {
                    let response = AuthLoginError{
                        message: "Error trying to login!",
                        error: "Invalid email or password."
                    };

                    return HttpResponse::NotFound()
                            .content_type(ContentType::json())
                            .json(response);
                }
            }
        },
        Err(error) => {
            return HttpResponse::BadRequest().content_type(ContentType::json()).json(error)
        }
    }
}
    


/// Endpoint que valida e retorna credenciais carregadas no token JWT passado no header da requisição
pub async fn validate_token(req: HttpRequest) -> impl Responder {
    let token = req.headers().get("authorization").unwrap();
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

/// Endpoints de autenticação
pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/auth")
        .route("/login", web::post().to(login))
        .service(web::scope("")
            .route("/validateToken", web::post().to(validate_token))
            .wrap(from_fn(auth_middleware))
        )
    );
}