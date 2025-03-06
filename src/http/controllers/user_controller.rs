use actix_web::{http::header::ContentType, middleware::from_fn, web::{self, ServiceConfig}, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use diesel::{ ExpressionMethods, QueryDsl, SelectableHelper, TextExpressionMethods};
use diesel_async::RunQueryDsl;
use crate::{database::db::get_connection, http::{middleware::auth_middleware::auth_middleware, requests::user::{user_filter_request::UserFilterRequest, user_store_request::UserStoreRequest, user_update_request::UserUpdateRequest}, responses::{auth::auth_login_response::AuthLoginError, user::{user_delete_response::{UserDeleteError, UserDeleteResponse}, user_store_response::{UserStoreError, UserStoreResponse}}}}, model::user::{user::User, user_dto::UserDTO}, schema::users::{self}, services::auth::decode_jwt};
use crate::schema::users::dsl::*;

pub async fn index(query_params: web::Query<UserFilterRequest>) -> impl Responder {

    let conn = &mut get_connection().await.unwrap();

    let mut query = users.into_boxed();

    if let Some(id_query) = query_params.0.id {
        query = query.filter(users::id.eq(id_query));
    }

    if let Some(name_query) = query_params.0.name {
        query = query.filter(users::name.like(format!("%{}%", name_query)));
    }

    if let Some(email_query) = query_params.0.email {
        query = query.filter(users::email.like(format!("%{}%", email_query)));
    }

    if let Some(page_query) = query_params.0.page {
        
        let per_page = match query_params.0.per_page {
            Some(per_page) => per_page,
            None => 5,
        };
        
        let offset_num = ((page_query - 1) * per_page) as i64;
        query = query.limit(per_page as i64).offset(offset_num);
    }


    let results = query.select(User::as_select()).get_results::<User>(conn).await;

    match results {
        Ok(query_users) => HttpResponse::Ok().content_type(ContentType::json()).json(query_users),
        Err(_err) => HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .json("Error querying users on DB!")
    }
}

pub async fn store(body: web::Json<UserStoreRequest>) -> impl Responder {
    let mut data = body.into_inner();
    
    data.password = match bcrypt::hash(&data.password, 10) {
        Ok(password_data) => password_data,
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

pub async fn update(path: web::Path<(i32, )>, body: web::Json<UserUpdateRequest>) -> impl Responder {

    let conn = &mut get_connection().await.unwrap();

    let find_user = users::table
        .filter(users::id.eq(path.into_inner().0))
        .select(User::as_select())
        .get_result::<User>(conn)
        .await;

    let date_now = Utc::now();

    match find_user {
        Ok(user_found) => {
            match bcrypt::verify(&body.old_password, &user_found.password) {
                Ok(result) => {
                    if result == true {
                        let new_password = match bcrypt::hash(body.new_password.clone(), 10){
                            Ok(password_data) => password_data,
                            Err(_err) => panic!("Error while bcrypt password")
                        };
    
                        let new_updated_user = UserDTO {
                            name: body.name.clone(),
                            email: body.email.clone(),
                            password: new_password,
                            created_at: user_found.created_at,
                            updated_at: Some(date_now),
                            deleted_at: None
                        };
        
                        let updated_user = diesel::update(
                            users::table.filter(users::id.eq(user_found.id))
                        ).set(new_updated_user).get_result::<User>(conn).await;
        
                        match updated_user {
                            Ok(user) => {
                                HttpResponse::Ok()
                                    .content_type(ContentType::json())
                                    .json(user)
                            },
                            Err(_error) => {
                                HttpResponse::BadRequest()
                                .content_type(ContentType::json())
                                .json("Something gone wrong updating user!")
                            }
                        }
                
                    } else {
                        HttpResponse::BadRequest()
                            .content_type(ContentType::json())
                            .json("Password confirmation gone wrong!")
                    }
                },
                Err(_err) => HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .json("Password confirmation gone wrong!")
            }
            

        },
        Err(_err) => HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .json("User not found!")
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
            .service(web::scope("")
                .route("/deleteMyAccount",web::delete().to(delete_my_account))
                .route("/update/{id}", web::put().to(update))
                .route("/index", web::get().to(index))
                .wrap(from_fn(auth_middleware))
            )
    );

}