use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use pethotel_api::http::controllers::user_controller;

#[get("/")]
async fn check_running() -> impl Responder {
    HttpResponse::Ok().body("Bem vindo ao PetHotel API!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let app = App::new()
            .service(check_running)
            .configure(user_controller::config);
        return app;
    })
    .workers(4)
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}