use actix_web::{
    body::MessageBody, dev::{ServiceRequest, ServiceResponse}, middleware::Next, Error
};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lazy_static::lazy_static;
use redis::AsyncCommands;
use dotenvy_macro::dotenv;

lazy_static! {
    static ref REDIS_URL: String = {
        format!("redis://:{}@{}", dotenv!("REDIS_PASSWORD"), dotenv!("REDIS_ADDRESS"))
    };

    static ref TIME_BLOCK_IP: u64 = {
        let time_str = dotenv!("TIME_BLOCK_IP");
        let time_block: Result<u64, _> = time_str.parse();
        
        match time_block {
            Ok(time) => time,
            Err(_) => 18000
        }
    };

    static ref MAX_REQUESTS_TRIES_ALLOWED: u32 = {
        let max_reqs_str = dotenv!("MAX_REQUESTS_TRIES_ALLOWED");
        let max_reqs_to_block: Result<u32, _> = max_reqs_str.parse();
        
        match max_reqs_to_block {
            Ok(reqs) => reqs,
            Err(_) => 15
        }
    };
}

use crate::{database::db::get_connection, http::GenericError, model::user::user::User, schema::users, services::auth::decode_jwt};

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {

    if let Some(token) = req.headers().get("authorization") {

        let client = &mut redis::Client::open(REDIS_URL.to_owned())
                    .unwrap()
                    .get_multiplexed_tokio_connection()
                    .await
                    .unwrap();

        match decode_jwt(token.to_str().expect("Error casting headervalue to &str")) {
            Ok(claim) => {
                let conn = &mut get_connection().await.unwrap();
                
                let find_user = users::table
                    .filter(users::email.eq(&claim.sub))
                    .select(User::as_select())
                    .get_result::<User>(conn)
                    .await;

                match find_user {
                    Ok(_user) => next.call(req).await,
                    Err(_error) => {
                        let error_response = GenericError {
                            message: "No user Logged!",
                            error: "Some error raised on server side!"
                        };
        
                        let error = Err(error_response);
                        return error.map_err(|e| actix_web::error::ErrorBadRequest(e))?;
                    }
                }
                
            },
            Err(_error) => {
                let ip_address = req.connection_info().peer_addr().unwrap().to_owned();

                
                let has_ip = client.get::<&str, u32>(&ip_address).await;

                match has_ip {
                    Ok(times) => {
                        if times <= MAX_REQUESTS_TRIES_ALLOWED.to_owned() {
                            let _ = client.set_ex::<&str, u32, String>(
                                &ip_address, 
                                times + 1, 
                                TIME_BLOCK_IP.to_owned())
                                .await
                                .unwrap();
                            let error_response = GenericError {
                                message: "Unathorized user!",
                                error: "Your Token do not match with our API!"
                            };
                
                            let error = Err(error_response);
                            return error.map_err(|e| actix_web::error::ErrorUnauthorized(e))?;
                        } else {
                            return next.call(req).await;
                        }
                    },
                    Err(redis_error) => {
                        let causer = redis_error.code();
                        match causer {
                            Some(_error) => {
                                let error_response = GenericError {
                                    message: "Server side error!",
                                    error: "Some error raised on server side!"
                                };
                    
                                let error = Err(error_response);
                                return error.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
                            }
                            None => {
                                let error_response = GenericError {
                                    message: "Unathorized user!",
                                    error: "Your Token do not match with our API!"
                                };
                    
                                let error = Err(error_response);
                                
                                client.set_ex::<&str, u32, String>(&ip_address, 1, TIME_BLOCK_IP.to_owned()).await.unwrap();
                                return error.map_err(|e| actix_web::error::ErrorUnauthorized(e))?;
                            }
                        }
                    }
                }
            }
        }
    } else {
        let user_not_found_response = GenericError {
            message: "No user Logged!",
            error: "authorization Header not found."
        };
        let error = Err(user_not_found_response);
        return error.map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    }
}