
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    Error,
};


pub async fn brute_force_wall_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let requisicao = req.connection_info().peer_addr().unwrap().to_owned();
    println!("{:#?}", requisicao);
    next.call(req).await
}