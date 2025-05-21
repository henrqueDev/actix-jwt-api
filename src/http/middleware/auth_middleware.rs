use actix_web::{
    body::MessageBody, dev::{ServiceRequest, ServiceResponse}, middleware::Next, Error
};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lazy_static::lazy_static;
use redis::AsyncCommands;
use dotenvy_macro::dotenv;
use jsonwebtoken::errors::ErrorKind::ExpiredSignature;

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

use crate::{database::db::get_connection, models::user::user::User, schema::users, services::auth::decode_jwt};

use super::{bad_request_response, internal_server_error_response, unauthorized_response};

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {

    if let Some(token) = req.headers().get("authorization") {


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
                        return Err(bad_request_response(
                            req,
                            "Some error raised on server side!",
                            "No user Logged!",
                            None
                        ).await.unwrap());
                    }
                }
                
            },
            Err(error) => {
                let the_error = error.into_kind();
                
                if the_error == ExpiredSignature {
                    return Err(unauthorized_response(
                        req,
                        "Your Token expired, please login again.", 
                        "Expired token!", 
                        None
                    ).await.unwrap());
                }

                let ip_address = req.connection_info().peer_addr().unwrap().to_owned();

                let client = &mut redis::Client::open(REDIS_URL.to_owned())
                    .unwrap()
                    .get_multiplexed_tokio_connection()
                    .await
                    .unwrap();
                
                let has_ip = client.get::<&str, u32>(&ip_address).await;

                match has_ip {
                    Ok(times) => {
                        if times <= MAX_REQUESTS_TRIES_ALLOWED.to_owned() {
                            return Err(unauthorized_response(
                                req,
                                "Your Token do not match with our API!",
                                "Unathorized user!", 
                                Some(10)
                            ).await.unwrap());
                        } else {
                            return next.call(req).await;
                        }
                    },
                    Err(redis_error) => {
                        let causer = redis_error.code();
                        match causer {
                            Some(_error) => {
                                return Err(internal_server_error_response(
                                    "Some error raised on server side!", 
                                    "Server side error!"
                                ).await.unwrap());
                            }
                            None => {
                                return Err(unauthorized_response(
                                    req, 
                                    "Your Token do not match with our API!", 
                                    "Unathorized user!", 
                                    Some(10)
                                ).await.unwrap());
                            }
                        }
                    }
                }
            }
        }
    } else {
        return Err(bad_request_response(
            req,
            "Authorization Header not found.",
            "No user Logged!",
            None
        ).await.unwrap());
    }
}