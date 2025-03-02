use actix_web::{http::header::ContentType, middleware::from_fn, web::{self, ServiceConfig}, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use diesel::{ ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use crate::{database::db::get_connection, http::{middleware::auth_middleware::auth_middleware, requests::user::user_store_request::UserStoreRequest, responses::{auth::auth_login_response::AuthLoginError, user::{user_delete_response::{UserDeleteError, UserDeleteResponse}, user_store_response::{UserStoreError, UserStoreResponse}}}}, model::user::{user::User, user_dto::UserDTO}, schema::users, services::auth::decode_jwt};

pub async fn store(body: web::Json<UserStoreRequest>) -> impl Responder {
    let mut data = body.into_inner();
    
    data.password = match bcrypt::hash(&data.password, 10) {
        Ok(password) => password,
        Err(_err) => panic!("Error while bcrypt password")
    };

    let date_now = Utc::now();

    let new_user = UserDTO{
        name: data.name, 
        email: data.email.clone(),
        password: data.password,
        created_at: Some(date_now), 
        updated_at: Some(date_now), 
        deleted_at: None
    };

    let conn = &mut get_connection().await.unwrap();
    let create_user = diesel::insert_into(users::table)
        .values(new_user)
        .execute(conn)
        .await;

    match create_user {
        Ok(_s) => {

            let user_created = users::table
                .filter(users::email.eq(data.email))
                .select(User::as_select())
                .get_result::<User>(conn)
                .await
                .expect("Error trying get user stored");

            let response = UserStoreResponse{
                message: String::from("User stored successfuly!"),
                user: user_created
            };

            return HttpResponse::Ok()
                .content_type(ContentType::json())
                .json(response);
        },
        Err(error) => {

            let response = UserStoreError{
                message: String::from("Error while trying store User!"),
                error: error.to_string()
            };

            return HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .json(response);
        }
    }
}

pub async fn delete_my_account(req: HttpRequest) -> impl Responder{
    if let Some(token) = req.headers().get("Authorization") {

        match decode_jwt(token.to_str().expect("Error casting headervalue to &str")) {
            Ok(claim) => {

                let conn = &mut get_connection().await.unwrap();
                
                let user_delete = diesel::delete(
                    users::table.filter(users::email.eq(&claim.sub))
                ).execute(conn).await;

                match user_delete {
                    Ok(rows) => {
                        if rows > 0 {
                            let response = UserDeleteResponse{
                                message: String::from("Your user was deleted successfully!"),
                                email: claim.sub
                            };
            
                            return HttpResponse::Ok()
                                .content_type(ContentType::json())
                                .json(response);
                        } else {
                            let message = String::from("No user was deleted because this user does not exist!");
                            let error = String::from("Your user does not exist in our database!");

                            let error_json = UserDeleteError {message, error};
        
                            return HttpResponse::NotFound()
                                .content_type(ContentType::json())
                                .json(error_json);
                        }
                    },
                    Err(error) => {
                        let message = format!("Some error raised deleting user");

                        let error_json = UserDeleteError {message, error: error.to_string()};
        
                        return HttpResponse::InternalServerError()
                            .content_type(ContentType::json())
                            .json(error_json);
                    }
                };
            },
            Err(_error) => {
                let error = String::from("Cannot decode JWT: your auth value may not be a Json Web Token.");

                let error_json = AuthLoginError {message: error};

                return HttpResponse::BadRequest()
                    .content_type(ContentType::json())
                    .json(error_json);
                
            }
        }
    } else {
        let error = String::from("No JWT Token was provided.");
        let error_json = AuthLoginError {message: error};

        return HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .json(error_json);
    }
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
web::scope("/users")
            .route("/store", web::post().to(store))
            .route("/deleteMyAccount", web::delete().to(delete_my_account))
            .wrap(from_fn(auth_middleware))
    );
}