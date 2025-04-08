use actix_web::{
    body::MessageBody, dev::{ServiceRequest, ServiceResponse}, middleware::Next, Error
};
use lazy_static::lazy_static;
use redis::AsyncCommands;
use dotenvy_macro::dotenv;

lazy_static! {
    static ref REDIS_URL: String = {
        format!("redis://:{}@{}", dotenv!("REDIS_PASSWORD"), dotenv!("REDIS_ADDRESS"))
    };
}

use crate::http::GenericError;

pub async fn brute_force_wall_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let client = &mut redis::Client::open(REDIS_URL.to_owned())
        .unwrap()
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    let ip_address = req.connection_info().peer_addr().unwrap().to_owned();

    if let Ok(times) = client.get::<&str, u32>(&ip_address).await {
        if times >= 5 {
            let error_response = GenericError {
                message: "Too many requests! Your IP its blocked for 5 hours!",
                error: "Too many requests."
            };

            let error = Err(error_response);

            return error.map_err(|e| actix_web::error::ErrorTooManyRequests(e))?;
        } else {
            return next.call(req).await;
        }
    } else {
        return next.call(req).await;
    }
        
}