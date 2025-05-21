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
    
    static ref MAX_REQUESTS_TRIES_ALLOWED: u32 = {
        let max_reqs_str = dotenv!("MAX_REQUESTS_TRIES_ALLOWED");
        let max_reqs_to_block: Result<u32, _> = max_reqs_str.parse();
        
        match max_reqs_to_block {
            Ok(reqs) => reqs,
            Err(_) => 15
        }
    };
}

use super::too_many_requests_error_response;

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
        if times >= MAX_REQUESTS_TRIES_ALLOWED.to_owned() {
            return Err(too_many_requests_error_response(
                "Too many requests.",
                "Too many requests! Your IP its blocked!"
                ).await.unwrap()
            )
        } else {
            return next.call(req).await;
        }
    } else {
        return next.call(req).await;
    }
        
}