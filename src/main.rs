use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use actix_jwt_api::{database::db::get_connection_sync, http::controllers::{auth_controller, user_controller}};
use dotenv_codegen::dotenv;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[get("/")]
async fn check_running() -> impl Responder {
    HttpResponse::Ok().body(format!("Welcome to {:#?}", dotenv!("APP_NAME")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let mut connection =  get_connection_sync().unwrap();
    
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Error migrating pending requests");

    let app_addr = dotenv!("APP_ADDRESS");
    let app_port: u16 = dotenv!("APP_PORT")
        .parse()
        .expect("PORT must be a valid integer");
    
    HttpServer::new(|| {
        let app = App::new()
            .service(check_running)
            .configure(user_controller::config)
            .configure(auth_controller::config);
        
        return app;
    })
    .workers(4)
    .bind((app_addr, app_port))?
    .run()
    .await
}