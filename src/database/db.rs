use std::env;

use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy_macro::dotenv;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub async fn get_connection() -> Result<AsyncPgConnection, ConnectionError> {
        let env_test = env::args()
            .into_iter()
            .find(|x| *x == "test_env".to_owned());
        
        let url = match env_test {
            Some(_env_test) => dotenv!("DATABASE_TEST_URL"),
            None => dotenv!("DATABASE_URL")
        };

        return AsyncPgConnection::establish(url)
        .await;
    
}

pub fn run_pending_migrations_db() -> PgConnection {

    let url_api = dotenv!("DATABASE_URL");
    let url_test = dotenv!("DATABASE_TEST_URL");

    let mut connection = PgConnection::establish(url_api).unwrap();
    let mut connection_test = PgConnection::establish(url_test).unwrap();
    
    connection_test
        .run_pending_migrations(MIGRATIONS)
        .expect("Error migrating pending requests");

    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Error migrating pending requests");
    
    return connection;
}