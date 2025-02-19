use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use pethotel_api::http::controllers::{auth_controller, user_controller};
use dotenv_codegen::dotenv;

#[get("/")]
async fn check_running() -> impl Responder {
    HttpResponse::Ok().body("Bem vindo ao PetHotel API!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

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