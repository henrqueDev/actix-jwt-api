use std::env;

use actix_web::{get, middleware::from_fn, App, HttpResponse, HttpServer, Responder};
use actix_jwt_api::{database::db::run_pending_migrations_db, http::{controllers::{auth_controller, email_controller, product_controller, user_controller}, middleware::brute_force_wall_middleware::brute_force_wall_middleware}};
use dotenvy_macro::dotenv;

#[get("/")]
async fn check_running() -> impl Responder {
    HttpResponse::Ok().body(format!("Welcome to {:#?}", dotenv!("APP_NAME")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_BACKTRACE", "1");

    run_pending_migrations_db();

    let app_addr = dotenv!("APP_ADDRESS");
    let app_port: u16 = dotenv!("APP_PORT")
        .parse()
        .expect("PORT must be a valid integer");
    
    HttpServer::new(|| {
        let app = App::new()
            .service(check_running)
            .configure(user_controller::config)
            .configure(email_controller::config)
            .configure(auth_controller::config)
            .configure(product_controller::config)
            .wrap(from_fn(brute_force_wall_middleware));
            
        return app;
    })
    .workers(4)
    .bind((app_addr, app_port))?
    .run()
    .await
}