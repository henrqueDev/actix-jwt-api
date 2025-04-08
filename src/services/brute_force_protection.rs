use actix_web::HttpRequest;
use lazy_static::lazy_static;
use redis::AsyncCommands;
use dotenvy_macro::dotenv;

lazy_static! {
    static ref REDIS_URL: String = {
        format!("redis://:{}@{}", dotenv!("REDIS_PASSWORD"), dotenv!("REDIS_ADDRESS"))
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
        if *times < 5 {
            let _ = client.set_ex::<&str, u32, String>(&ip_address, times + 1, 18000).await;
        }
    } else {
        let _ = client.set_ex::<&str, u32, String>(&ip_address, 1, 18000).await;    
    }
        
}