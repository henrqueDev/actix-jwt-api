use actix_web::HttpRequest;
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
            Err(_) => 360
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


pub async fn brute_force_protection(
    req: HttpRequest
) -> () {
    let client = &mut redis::Client::open(REDIS_URL.to_owned())
        .unwrap()
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    let ip_address = req.connection_info().peer_addr().unwrap().to_owned();
    let ip_key_val = client.get::<&str, u32>(&ip_address).await;

    if let Ok(times) =  &ip_key_val {
        if *times < MAX_REQUESTS_TRIES_ALLOWED.to_owned() {
            let _ = client.set_ex::<&str, u32, String>(&ip_address, times + 1, TIME_BLOCK_IP.to_owned()).await;
        }
    } else {
        let _ = client.set_ex::<&str, u32, String>(&ip_address, 1, TIME_BLOCK_IP.to_owned()).await;    
    }
        
}