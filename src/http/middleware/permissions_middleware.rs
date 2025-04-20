use actix_web::{
    body::MessageBody, dev::{ServiceRequest, ServiceResponse}, middleware::Next, Error
};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lazy_static::lazy_static;
use dotenvy_macro::dotenv;

lazy_static! {
    static ref REDIS_URL: String = {
        format!("redis://:{}@{}", dotenv!("REDIS_PASSWORD"), dotenv!("REDIS_ADDRESS"))
    };
}

use crate::{database::db::get_connection, http::GenericError, models::user::user::User, schema::users, services::{auth::decode_jwt, brute_force_protection::brute_force_protection}};

pub async fn permissions_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {

    if let Some(token) = req.headers().get("authorization") {


        match decode_jwt(token.to_str().expect("Error casting headervalue to &str")) {
            Ok(claim) => {
                let conn = &mut get_connection().await.unwrap();
                
                let find_user = users::table
                    .filter(users::email.eq(&claim.sub))
                    .select(User::as_select())
                    .get_result::<User>(conn)
                    .await;

                match find_user {
                    Ok(user) => {
                        if user.user_type_id == 1 {
                            return next.call(req).await;
                        } else {
                            let error_response = GenericError {
                                message: "User has no permission to use this feature!",
                                error: "User has no permission to access this route (its admin only)."
                            };
            
                            let error = Err(error_response);
                            return error.map_err(|e| actix_web::error::ErrorUnauthorized(e))?;
                        }
                    },
                    Err(_error) => {
                        let error_response = GenericError {
                            message: "No user Logged!",
                            error: "Some error raised on server side!"
                        };
        
                        let error = Err(error_response);
                        return error.map_err(|e| actix_web::error::ErrorBadRequest(e))?;
                    }
                }
                
            },
            Err(_error) => {
                
                let error_response = GenericError {
                    message: "Error decoding JWT Token!",
                    error: "Your JWT Token does not match to this API."
                };
                
                let error = Err(error_response);

                let request = req.request().clone();
                
                brute_force_protection(request).await;
                
                return error.map_err(|e| actix_web::error::ErrorBadRequest(e))?;
            }
        }
    } else {
        let user_not_found_response = GenericError {
            message: "No user Logged!",
            error: "authorization Header not found."
        };
        let error = Err(user_not_found_response);
        return error.map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    }
}