use actix_web::{dev::ServiceRequest, Error};

use crate::services::brute_force_protection::brute_force_protection;
use super::GenericError;

pub mod auth_middleware;
pub mod brute_force_wall_middleware;
pub mod permissions_middleware;

pub async fn bad_request_response(req: ServiceRequest, error: &'static str, message: &'static str, checked_brute_force: Option<i16>) -> Result<Error, Box<dyn std::error::Error + 'static>> {
    let user_not_found_response: GenericError<'static> = GenericError {
        message,
        error
    };

    if let Some(flag_points) = checked_brute_force {
        let request = req
            .request()
            .clone();
        brute_force_protection(request, Some(flag_points)).await;
    }

    let error = Err(user_not_found_response);
    
    return error.map_err(|e| actix_web::error::ErrorBadRequest(e))?;
}

pub async fn unauthorized_response(req: ServiceRequest, error: &'static str, message: &'static str, checked_brute_force: Option<i16>) -> Result<Error, Box<dyn std::error::Error + 'static>> {
    let user_not_found_response: GenericError<'static> = GenericError {
        message,
        error
    };

    if let Some(flag_points) = checked_brute_force {
        let request = req
            .request()
            .clone();
        brute_force_protection(request, Some(flag_points)).await;
    }

    let error = Err(user_not_found_response);
    
    return error.map_err(|e| actix_web::error::ErrorUnauthorized(e))?;
}


pub async fn internal_server_error_response(error: &'static str, message: &'static str) -> Result<Error, Box<dyn std::error::Error + 'static>> {
    let user_not_found_response: GenericError<'static> = GenericError {
        message,
        error
    };

    let error = Err(user_not_found_response);
    
    return error.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
}

pub async fn too_many_requests_error_response(error: &'static str, message: &'static str) -> Result<Error, Box<dyn std::error::Error + 'static>> {
    let user_not_found_response: GenericError<'static> = GenericError {
        message,
        error
    };

    let error = Err(user_not_found_response);
    
    return error.map_err(|e| actix_web::error::ErrorTooManyRequests(e))?;
}