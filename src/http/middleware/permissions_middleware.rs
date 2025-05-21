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

use crate::{database::db::get_connection, models::user::user::User, schema::users, services::auth::decode_jwt};

use super::{bad_request_response, unauthorized_response};

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
                            return Err(unauthorized_response(
                                req, 
                                "User has no permission to access this route (its admin only).",
                                "User has no permission to use this feature!",
                                None
                            ).await.unwrap());
                        }
                    },
                    Err(_error) => {
                        return Err(bad_request_response(
                            req,
                            "Some Error raised at server side.", 
                            "No user Logged!",
                            None
                        ).await.unwrap());
                    }
                }
                
            },
            Err(_error) => {
                return Err(bad_request_response(
                    req,
                    "Error decoding JWT Token!",
                    "Your JWT Token does not match to this API.",
                    Some(10)
                ).await.unwrap());
            }
        }
    } else {
        return Err(bad_request_response(
            req,
            "Authorization Header not found.", 
            "No user Logged!",
            None
        ).await.unwrap());
    }
}