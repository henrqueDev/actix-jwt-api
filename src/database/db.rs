use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection};

use dotenvy_macro::dotenv;

pub async fn get_connection() -> Result<AsyncPgConnection, ConnectionError> {
    let url = dotenv!("DATABASE_URL");
    
    return AsyncPgConnection::establish(url)
        .await;
}

pub fn get_connection_sync() -> Result<PgConnection, ConnectionError> {
    let url = dotenv!("DATABASE_URL");
    
    return PgConnection::establish(url);
}