use actix_web::{http::header::ContentType, middleware::from_fn, web::{self, ServiceConfig}, HttpResponse, Responder};
use chrono::Utc;
use diesel::{ ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use crate::{database::db::get_connection, http::{middleware::auth_middleware::auth_middleware, requests::user::user_store_request::UserStoreRequest, responses::user::user_store_response::{UserStoreError, UserStoreResponse}}, model::user::{user::User, user_dto::UserDTO}, schema::users};

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

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
web::scope("/users")
            .route("/store", web::post().to(store))
            .wrap(from_fn(auth_middleware))
    );
}