use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    Error,
};

use crate::services::auth::decode_jwt;

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {

    if let Some(token) = req.headers().get("Authorization") {

        match decode_jwt(token.to_str().expect("Error casting headervalue to &str")) {
            Ok(_claim) => {
                return next.call(req).await;
            },
            Err(_error) => {
                let error = Err("No user logged!");
                return error.map_err(|e| actix_web::error::ErrorBadRequest(e))?;
                
            }
        }
    } else {
        let error = Err("No user logged!");
        return error.map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    }
}