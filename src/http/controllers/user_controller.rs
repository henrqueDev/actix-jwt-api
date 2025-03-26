use actix_web::{ http::header::ContentType, middleware::from_fn, web::{self, ServiceConfig}, HttpRequest, HttpResponse, Responder};
use base32::{encode, Alphabet};
use chrono::Utc;
use diesel::{ ExpressionMethods, QueryDsl, SelectableHelper, TextExpressionMethods };
use diesel_async::RunQueryDsl;
use totp_rs::{Algorithm, Secret, TOTP};
use crate::{database::db::get_connection, http::{middleware::auth_middleware::auth_middleware, requests::user::{user_activate2fa_request::UserActivate2FARequest, user_filter_request::UserFilterRequest, user_store_request::UserStoreRequest, user_update_request::UserUpdateRequest}, responses::{auth::auth_login_response::AuthLoginError, user::{user_delete_response::{UserDeleteError, UserDeleteResponse}, user_enable2fa_response::UserEnable2FAResponse, user_index_response::UserIndexResponse, user_store_response::{UserStoreError, UserStoreResponse}, user_update_response::{UserUpdateError, UserUpdateResponse}}}, GenericError, GenericResponse}, model::user::{user::User, user_dto::{UserDTO, UserDTOMin}}, schema::users::{self}, services::auth::decode_jwt};
use crate::schema::users::dsl::*;
use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use rand::Rng;
use dotenv_codegen::dotenv;

pub async fn index(query_params: web::Query<UserFilterRequest>) -> impl Responder {

    let conn = &mut get_connection().await.unwrap();

    let mut query = users.into_boxed();

    // Aplicando filtros na consulta
    if let Some(id_query) = query_params.0.id {
        query = query.filter(users::id.eq(id_query));
    }

    if let Some(name_query) = query_params.0.name {
        query = query.filter(users::name.like(format!("%{}%", name_query)));
    }

    if let Some(email_query) = query_params.0.email {
        query = query.filter(users::email.like(format!("%{}%", email_query)));
    }

    // Filtro de paginação
    if let Some(page_query) = query_params.0.page {
        
        let per_page = match query_params.0.per_page {
            Some(per_page) => per_page,
            None => 5,
        };
        
        let offset_num = ((page_query - 1) * per_page) as i64;
        query = query.limit(per_page as i64).offset(offset_num);
    }

    // Consulta na tabela de users, retornando a struct UserDTOMin
    let results = query
        .select(UserDTOMin::as_select())
        .get_results::<UserDTOMin>(conn)
        .await;

    // Verificar se a consulta foi um sucesso (revisar tipagem do retorno de erro)
    match results {
        Ok(query_users) => {

            // Preparando dados para retornar para o client
            let users_response = UserIndexResponse {
                message: "Query users gone successfully!",
                users: query_users,
                current_page: query_params.0.page,
                per_page: query_params.0.per_page
            };

            // Resposta status 200
            HttpResponse::Ok().content_type(ContentType::json()).json(users_response)
        },
        Err(_err) => {


            // Erro interno no servidor durante consulta no banco
            let error_response = GenericError {
                message: "Error querying users on DB!",
                error: "Internal Server error while "
            };

            // Resposta com status 500
            HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .json(error_response)
        }
    }
}

pub async fn store(body: web::Json<UserStoreRequest>) -> impl Responder {
    let mut data = body.into_inner();
    
    // Gerar hash da senha passada no body
    data.password = match bcrypt::hash(&data.password, 10) {
        Ok(password_data) => password_data,
        Err(_err) => panic!("Error while bcrypt password")
    };

    // Pegar a hora atual
    let date_now = Utc::now();

    let new_user = UserDTO{
        name: data.name, 
        email: data.email.clone(),
        password: data.password,
        two_factor_secret: None,
        two_factor_recovery_code: None,
        two_factor_confirmed_at: None,
        created_at: Some(date_now), 
        updated_at: Some(date_now), 
        deleted_at: None
    };

    let conn = &mut get_connection().await.unwrap();
    
    // Query para criar o usuário
    let create_user = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(conn)
        .await;

    match create_user {
        Ok(user_created) => {

            // Preparar dados da resposta
            let response = UserStoreResponse{
                message: "User stored successfuly!",
                user: user_created
            };

            // Resposta com status 200
            return HttpResponse::Ok()
                .content_type(ContentType::json())
                .json(response);
        },
        Err(error) => {
            let error_msg = error.to_string();

            // Preparar dados do erro interno para a resposta
            let response = UserStoreError{
                message: "Error while trying store User!",
                error: &error_msg
            };

            // Retornar dados com status 500
            return HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .json(response);
        }
    }
}

pub async fn update(path: web::Path<i32>, body: web::Json<UserUpdateRequest>) -> impl Responder {

    let conn = &mut get_connection().await.unwrap();

    // Consulta o usuario no banco pelo ID passado no path do endpoint
    let find_user  = users::table
        .filter(users::id.eq(path.into_inner()))
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
                            two_factor_secret: user_found.two_factor_secret,
                            two_factor_recovery_code: user_found.two_factor_recovery_code,
                            two_factor_confirmed_at: user_found.two_factor_confirmed_at,
                            created_at: user_found.created_at,
                            updated_at: Some(date_now),
                            deleted_at: user_found.deleted_at
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
                                message: "Your user was deleted successfully!",
                                email: &claim.sub
                            };
            
                            return HttpResponse::Ok()
                                .content_type(ContentType::json())
                                .json(response);
                        } else {

                            let error_json = UserDeleteError {
                                message: "User was not deleted!", 
                                error: "Your user may not exists in our database or server was not able to query righlty!"
                            };
        
                            return HttpResponse::NotFound()
                                .content_type(ContentType::json())
                                .json(error_json);
                        }
                    },
                    Err(error) => {
                        let error_msg = error.to_string();

                        let error_json = UserDeleteError {
                            message: "Some error raised deleting user", 
                            error: &error_msg
                        };
        
                        return HttpResponse::InternalServerError()
                            .content_type(ContentType::json())
                            .json(error_json);
                    }
                };
            },
            Err(_error) => {
                let error = String::from("Cannot decode JWT: your auth value may not be a Json Web Token.");

                let error_json = AuthLoginError {message: &error};

                return HttpResponse::BadRequest()
                    .content_type(ContentType::json())
                    .json(error_json);
                
            }
        }
    } else {
        let error_json = GenericError {
            message: "Error trying delete account!",
            error: "No JWT Token was provided."
        };

        return HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .json(error_json);
    }
}

pub async fn enable_2fa(req: HttpRequest) -> impl Responder {
    let token = req.headers().get("Authorization").unwrap();

    match decode_jwt(token.to_str().expect("Error casting headervalue to &str")) {
        Ok(claim) => {
            
            let conn = &mut get_connection().await.unwrap();

            let user = users::table
                .filter(users::email.eq(claim.sub))
                .select(User::as_select())
                .get_result::<User>(conn)
                .await
                .expect("User does not exists!");

            let mut rng = rand::rng();
            let mut random_bytes = [0u8; 32];
            rng.fill(&mut random_bytes);

            let app_name = dotenv!("APP_NAME");
        
            let random_code = BASE64_STANDARD_NO_PAD.encode(&random_bytes);
            let base_32_code = encode(Alphabet::Rfc4648 { padding: true }, random_code.as_bytes());

            let totp = TOTP::new(
                Algorithm::SHA512,
                6,
                1,
                30,
                Secret::Encoded(base_32_code).to_bytes().unwrap(),
                Some(app_name.to_string()),
                user.email.clone()
            ).unwrap();

            let qrcode_base64 = totp.get_qr_base64().unwrap();
            let setup_key = totp.get_secret_base32();
            let date_now = Utc::now();

            let new_updated_user = UserDTO {
                name: user.name,
                email: user.email,
                password: user.password,
                two_factor_secret: Some(setup_key.clone()),
                two_factor_recovery_code: user.two_factor_recovery_code,
                two_factor_confirmed_at: user.two_factor_confirmed_at,
                created_at: user.created_at,
                updated_at: Some(date_now),
                deleted_at: user.deleted_at
            };

            diesel::update(users::table.filter(users::id.eq(user.id)))
                .set(new_updated_user)
                .execute(conn)
                .await
                .expect("Error setting 2FA code for user!");
            
            let response = UserEnable2FAResponse {
                message: "QR Code and Config Key generated! Confirm code at /user/activate-2fa",
                qrcode: &qrcode_base64,
                config_code: &setup_key
            }; 

            HttpResponse::Ok().content_type(ContentType::json()).json(response)
        },
        Err(_err) => {
            let error_response = GenericError {
                message: "No user Logged!",
                error: "Invalid Authorization token."
            };

            HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .json(error_response)
        }
    }

} 

pub async fn activate_2fa(req: HttpRequest, body: web::Json<UserActivate2FARequest>) -> impl Responder {
    let token = req.headers().get("Authorization").unwrap();

    
    match decode_jwt(token.to_str().expect("Error casting headervalue to &str")) {
        Ok(claim) => {
            
            let conn = &mut get_connection().await.unwrap();

            let user = users::table
                .filter(users::email.eq(claim.sub))
                .select(User::as_select())
                .get_result::<User>(conn)
                .await
                .expect("User does not exists!");

            match user.two_factor_secret {
                    Some(secret) => {
                        let date_now = Utc::now();
                        let app_name = dotenv!("APP_NAME");

                        let totp = TOTP::new(
                            Algorithm::SHA512,
                            6,
                            1,
                            30,
                            Secret::Encoded(secret.clone()).to_bytes().unwrap(),
                            Some(app_name.to_string()),
                            user.email.clone()
                        ).unwrap();

                        let seconds_now = ((Utc::now().timestamp_millis()) / 1000) as u64;

                        if totp.check(body.code.as_str(), seconds_now) == true {
                            let new_updated_user = UserDTO {
                                name: user.name,
                                email: user.email,
                                password: user.password,
                                two_factor_secret:  Some(secret),
                                two_factor_recovery_code: user.two_factor_recovery_code,
                                two_factor_confirmed_at: Some(date_now),
                                created_at: user.created_at,
                                updated_at: Some(date_now),
                                deleted_at: user.deleted_at
                            };
                
                            diesel::update(users::table.filter(users::id.eq(user.id)))
                                .set(new_updated_user)
                                .execute(conn)
                                .await
                                .expect("Error setting 2FA code for user!");
                            
                            let response = GenericResponse {
                                message: "2FA setted up successfully!"
                            }; 
                
                            HttpResponse::Ok().content_type(ContentType::json()).json(response)
                        } else {
                            HttpResponse::Unauthorized()
                                .content_type(ContentType::json())
                                .json("User failed the 2FA challenge code!".to_string())
                        }

                    },
                    None => {
                        HttpResponse::Unauthorized()
                            .content_type(ContentType::json())
                            .json("User did not request 2FA challenge!".to_string())
                    }
            }

        },
        Err(_err) => {
            let error_response = GenericError {
                message: "No user Logged!",
                error: "Invalid Authorization token."
            };

            HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .json(error_response)
        }
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
                .route("/enable-2fa", web::get().to(enable_2fa))
                .route("/activate-2fa", web::post().to(activate_2fa))
                .wrap(from_fn(auth_middleware))
            )
    );

}