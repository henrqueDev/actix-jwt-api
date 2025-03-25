use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    Error,
};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

use crate::{database::db::get_connection, http::GenericError, model::user::user::User, schema::users, services::auth::decode_jwt};

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {

    if let Some(token) = req.headers().get("Authorization") {

        match decode_jwt(token.to_str().expect("Error casting headervalue to &str")) {
            Ok(claim) => {
                let conn = &mut get_connection().await.unwrap();
                
                let find_user = users::table
                    .filter(users::email.eq(&claim.sub))
                    .select(User::as_select())
                    .get_result::<User>(conn)
                    .await;

                match find_user {
                    Ok(_user) => next.call(req).await,
                    Err(_error) => {
                        let error_response = GenericError {
                            message: "No user Logged!",
                            error: Some("Some error raised on server side!")
                        };
        
                        let error = Err(error_response);
                        return error.map_err(|e| actix_web::error::ErrorBadRequest(e))?;
                    }
                }
                
            },
            Err(_error) => {
                
                let error_response = GenericError {
                    message: "No user Logged!",
                    error: Some("Some error raised on server side!")
                };

                let error = Err(error_response);
                return error.map_err(|e| actix_web::error::ErrorBadRequest(e))?;
                
            }
        }
    } else {
        let user_not_found_response = GenericError {
            message: "No user Logged!",
            error: Some("Authorization Header not found.")
        };
        let error = Err(user_not_found_response);
        return error.map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    }
}