use lazy_static::lazy_static;
use dotenvy_macro::dotenv;
use redis::{AsyncCommands, RedisError};
use redis::{FromRedisValue, ToRedisArgs};

lazy_static! {
    static ref REDIS_URL: String = {
        format!("redis://:{}@{}", dotenv!("REDIS_PASSWORD"), dotenv!("REDIS_ADDRESS"))
    };
}

pub async fn cache_del_key<'a, 
    K: ToRedisArgs + Send + Sync + 'a,
    RV: FromRedisValue
> (key: K) -> Result<RV, RedisError> {
    let redis_client = &mut redis::Client::open(REDIS_URL.to_owned())
        .unwrap()
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    return redis_client.del(key).await;
}

pub async fn cache_set_key<'a, 
    K: ToRedisArgs + Send + Sync + 'a,
    V: ToRedisArgs + Send + Sync + 'a,
    RV: FromRedisValue
> (key: K, value: V, exp_time: u64) -> Result<RV, RedisError> {
    let redis_client = &mut redis::Client::open(REDIS_URL.to_owned())
        .unwrap()
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    return redis_client.set_ex::<K, V, RV>(key, value, exp_time).await;
}

pub async fn cache_get_key<'a, 
    K: ToRedisArgs + Send + Sync + 'a,
    RV: FromRedisValue
> (key: K) -> Result<RV, RedisError> {
    let redis_client = &mut redis::Client::open(REDIS_URL.to_owned())
        .unwrap()
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    return redis_client.get::<K, RV>(key).await;
}