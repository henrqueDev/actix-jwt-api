use actix_web::{ http::header::ContentType, middleware::from_fn, web::{self, ServiceConfig}, HttpRequest, HttpResponse, Responder};
use base32::{encode, Alphabet};
use chrono::Utc;
use diesel::{ ExpressionMethods, QueryDsl, SelectableHelper, TextExpressionMethods };
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use lazy_static::lazy_static;
use lettre::{message::{header, SinglePart}, transport::smtp::authentication::{Credentials, Mechanism}, Message, SmtpTransport, Transport};
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::{ContextV7, Timestamp, Uuid};
use validator::Validate;
use crate::{database::db::get_connection, http::{middleware::auth_middleware::auth_middleware, requests::user::{user_activate2fa_request::UserActivate2FARequest, user_activate_request::UserActivateRequest, user_filter_request::UserFilterRequest, user_resend_activation_hash_request::UserResendActivationHashRequest, user_store_request::UserStoreRequest, user_update_request::UserUpdateRequest}, responses::{auth::auth_login_response::AuthLoginError, email::email_sent_response::EmailSendError, user::{user_delete_response::{UserDeleteError, UserDeleteResponse}, user_enable2fa_response::UserEnable2FAResponse, user_index_response::UserIndexResponse, user_store_response::{UserStoreError, UserStoreResponse}, user_update_response::{UserUpdateError, UserUpdateResponse}}}, GenericError, GenericResponse}, models::user::{user::User, user_dto::{UserDTO, UserDTOMin}}, schema::users::{self}, services::{auth::decode_jwt, google_oauth2::refresh_oauth2_google, redis_client::{cache_del_key, cache_get_key, cache_set_key}}};
use crate::schema::users::dsl::*;
use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use rand::Rng;
use dotenvy_macro::dotenv;

lazy_static! {
    static ref BCRYPT_COST: u32 = dotenv!("BCRYPT_COST").parse().unwrap();
}

/// Endpoint para consulta de usuários com filtros opcionais
pub async fn index(query_params: web::Query<UserFilterRequest>) -> impl Responder {

    let conn = &mut get_connection().await.unwrap();

    let mut query = users.into_boxed();

    // Aplicando filtros na consulta (Revisar o framework)
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
            None => 5, // per_page padrão é 5
        };
        
        let offset_num = ((page_query - 1) * per_page) as i64;
        query = query.limit(per_page as i64).offset(offset_num);
    }

    // Apenas listar usuarios não excluidos
    query = query.filter(users::deleted_at.is_null());

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
            return HttpResponse::Ok().content_type(ContentType::json()).json(users_response);
        },
        Err(_err) => {
            

            // Erro interno no servidor durante consulta no banco
            let error_response = GenericError {
                message: "Error querying users on DB!",
                error: "Internal Server error while "
            };

            // Resposta com status 500
            return HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .json(error_response);
        }
    }
}

/// Endpoint para cadastro de novos usuários
pub async fn store(body: web::Json<UserStoreRequest>) -> impl Responder {
    let validate = body.validate();
    
    match validate {
        Ok(_) => {
            let data = body.into_inner();
    
            // Gerar hash da senha passada no body
            // data.password = match bcrypt::hash(&data.password, 10) {
            //     Ok(password_data) => password_data,
            //     Err(_err) => panic!("Error while bcrypt password")
            // };
        
            // Pegar a hora atual
            let date_now = Utc::now();
        
            let new_user = UserDTO{
                name: data.name, 
                email: data.email.clone(),
                password: None,
                two_factor_secret: None,
                two_factor_recovery_code: None,
                two_factor_confirmed_at: None,
                created_at: Some(date_now), 
                updated_at: Some(date_now), 
                deleted_at: None
            };
        
            let conn = &mut get_connection().await.unwrap();

            let check_user_already_exists = users::table
                .filter(users::email.eq(&new_user.email))
                .select(User::as_select())
                .get_result::<User>(conn)
                .await;

            match check_user_already_exists {
                Ok(_user) => {
                    let user_exists_res = UserStoreError {
                        message: "Email already being used!",
                        error: "A user with this same email already exists."
                    };

                    return HttpResponse::Conflict()
                        .content_type(ContentType::json())
                        .json(user_exists_res);
                },
                Err(_) => {
                    
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
                                user: user_created.clone()
                            };
        
                            let context = ContextV7::new();
                            let time_unix = Timestamp::now(&context).to_unix();
        
                            let uuid_user = Uuid::new_v7(
                                Timestamp::from_unix(&context,time_unix.0, time_unix.1)
                            );
        
                            let cache_uuid = cache_set_key::<&str, String, ()>(
                                &format!("user:{}:uuid_user_activate", &user_created.id),
                                uuid_user.to_string(),
                                86400
                            ).await;
        
                            match cache_uuid {
                                Ok(_) => {
                                    let message_email = format!("This is your Token for define your password {}", uuid_user.to_string());
        
                                    let email_text_body = SinglePart::builder()
                                        .header(header::ContentType::TEXT_PLAIN)
                                        .body(message_email);
                
                                    // Ler dados do usuário da aplicação (.env) e de quem vai receber o email
                                    let google_email = dotenv!("EMAIL");
                                    let user_receiver = user_created.email;
                                    
                                    // Criar o Email
                                    let email_singlepart = Message::builder()
                                        .from(google_email.parse().unwrap())
                                        .to(user_receiver.parse().unwrap())
                                        .subject("Ativar conta Logistica-APP")
                                        .singlepart(email_text_body).unwrap();

                                    let oauth_key = match cache_get_key::<&str, String>("GOOGLE_OAUTH2_KEY").await {
                                        Ok(access_token) => access_token,
                                        Err(_) => refresh_oauth2_google().await
                                    };
                
                                    // Resgatar as credenciais para conexão segura
                                    let creds = Credentials::new(google_email.to_owned(), oauth_key);
                
                                    // Construtor do algoritmo de transporte pelo serviço do Gmail
                                    let mailer = SmtpTransport::starttls_relay("smtp.gmail.com").expect("Error creating StartTLS Transport")
                                        .authentication(vec![Mechanism::Xoauth2])
                                        .credentials(creds)
                                        .build();
                
                                    // Enviar email e verificar se o envio deu certo
                                    match mailer.send(&email_singlepart) { // Resposta com status 200
                                        Ok(_) => return HttpResponse::Ok()
                                            .content_type(ContentType::json())
                                            .json(response),
                                        Err(e) => HttpResponse::BadGateway()
                                            .content_type(ContentType::json())
                                            .json(EmailSendError {
                                                message: "Error sending email!",
                                                error: &e.to_string()
                                            }),
                                    }
                                },
                                Err(error) => {
                                    let error_msg = error.to_string();
        
                                    // Preparar dados do erro interno para a resposta
                                    let response = GenericError {
                                        message: "Error while setting uuid key in cache!",
                                        error: &error_msg
                                    };
                                    
                                    // Retornar dados com status 500
                                    return HttpResponse::InternalServerError()
                                        .content_type(ContentType::json())
                                        .json(response);
                                }
                            }
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
            }
        },
        Err(error) => {
            return HttpResponse::BadRequest().content_type(ContentType::json()).json(error);
        }
    }
}

/// Endpoint para atualização de usuários
/// 
/// Revisar: Usuários podem modificar outro usuário, implementar middleware de permissões.
pub async fn update(path: web::Path<i32>, body: web::Json<UserUpdateRequest>) -> impl Responder {

    let validate = body.validate();

    match validate {
        Ok(_) => {
            let conn = &mut get_connection().await.unwrap();

            // Consulta o usuario no banco pelo ID passado no path da requisição
            let find_user  = users::table
                .filter(users::id.eq(path.into_inner()))
                .select(User::as_select())
                .get_result::<User>(conn)
                .await;

            match find_user {
                Ok(user_found) => {

                    // Preparar uma struct mutavel para atualizar os campos necessários 
                    let mut new_updated_user = UserDTO {
                        name: user_found.name,
                        email: user_found.email,
                        password: user_found.password.clone(),
                        two_factor_secret: user_found.two_factor_secret,
                        two_factor_recovery_code: user_found.two_factor_recovery_code,
                        two_factor_confirmed_at: user_found.two_factor_confirmed_at,
                        created_at: user_found.created_at,
                        updated_at: user_found.updated_at,
                        deleted_at: user_found.deleted_at
                    };
                    
                    // Verificar se o usuario quer atualizar a senha atual
                    match &body.old_password {
                        Some(old_password) => {
                            match &user_found.password {
                                Some(user_password) => {
                                    // Verificar se a senha corresponde com a que está no banco
                                    match bcrypt::verify(old_password, user_password) {
                                        Ok(result) => {
                    
                                            if result == true {
                    
                                                let new_password = match bcrypt::hash(body.new_password.clone().unwrap(), 10){
                                                    Ok(password_data) => password_data,
                                                    Err(_err) => panic!("Error while encrypt new password") // Revisar
                                                };
                    
                                                new_updated_user.password = Some(new_password);
                                        
                                            } else {
                                                let res_wrong_pass = UserUpdateError {
                                                    message: "Password confirmation gone wrong!",
                                                    error: "Old password is wrong! Try again."
                                                };

                                                return HttpResponse::Unauthorized()
                                                    .content_type(ContentType::json())
                                                    .json(res_wrong_pass);
                                            }
                                        },
                                        
                                        //Em caso de erro interno no processo de validação da senha antiga
                                        Err(err) => {
                                            
                                            let res_bcrypt_err = UserUpdateError {
                                                message: "Server wasnt able to parse old Password to confirm!",
                                                error: &err.to_string()
                                            };

                                            // Retornar resposta com status 500
                                            return HttpResponse::InternalServerError()
                                                .content_type(ContentType::json())
                                                .json(res_bcrypt_err);
                                        }
                                    };
                                },
                                None => {
                                    let res_err = UserUpdateError {
                                        message: "You hasn't set your user password yet!",
                                        error: "Error trying update user!"
                                    };
        
                                    return HttpResponse::Conflict()
                                    .content_type(ContentType::json())
                                    .json(res_err);
                                }
                            }
                            
                        },
                        None => {}, // Se não tiver o campo old_password, não faz nada
                    }

                    // Verificar se o campo de nome foi passado na requisição
                    match &body.name {
                        Some(new_name) => {new_updated_user.name = new_name.to_owned()},
                        None => {},
                    }

                    // Verificar se o campo de email foi passado na requisição
                    match &body.email {
                        Some(new_email) => {new_updated_user.email = new_email.to_owned()},
                        None => {},
                    }

                    // Setar o horario do update com o tempo universal coordenado sem offset de fuso horario (Revisar)
                    new_updated_user.updated_at = Some(Utc::now());

                    // Query para atualizar o usuario
                    let updated_user = diesel::update(
                        users::table.filter(users::id.eq(user_found.id))
                    ).set(new_updated_user).get_result::<User>(conn).await;

                    // Verificar resultado da query (sucesso ou erro interno)
                    match updated_user {

                        // Usuario atualizado
                        Ok(user) => {

                            // Resposta de sucesso
                            let res_updated_success = UserUpdateResponse {
                                message: "User updated successfully!",
                                user
                            };
                            
                            // Responder com status 200
                            return HttpResponse::Ok()
                                .content_type(ContentType::json())
                                .json(res_updated_success);
                        },
                        
                        // Erro interno ao executar a query
                        Err(_error) => {

                            let res_err = UserUpdateError {
                                message: "Error trying update user!",
                                error: "User was not found!"
                            };

                            return HttpResponse::NotFound()
                            .content_type(ContentType::json())
                            .json(res_err);
                        }
                    }

                },
                Err(_err) => {
                    let res_err = UserUpdateError {
                        message: "Error trying update user!",
                        error: "Internal server error raised while looking for user in Database"
                    };

                    return HttpResponse::InternalServerError()
                        .content_type(ContentType::json())
                        .json(res_err);
                }
            }
        },
        Err(error) => {
            return HttpResponse::BadRequest().content_type(ContentType::json()).json(error);
        }
    }

    
}

/// Endpoint para usuários deletarem a conta (exclusão lógica)
pub async fn delete_my_account(req: HttpRequest) -> impl Responder{
    if let Some(token) = req.headers().get("authorization") {

        match decode_jwt(token.to_str().expect("Error casting headervalue to &str")) {
            Ok(claim) => {

                let conn = &mut get_connection().await.unwrap();
                
                let user_delete = diesel::delete(
                    users::table.filter(users::email.eq(&claim.sub))
                ).execute(conn).await;

                match user_delete {
                    // Query rodou com sucesso
                    Ok(rows) => {

                        // Checar se alguma instancia no banco de dados foi afetada (revisar)
                        if rows > 0 {
                            let response = UserDeleteResponse{
                                message: "Your user was deleted successfully!",
                                email: &claim.sub
                            };
            
                            return HttpResponse::Ok()
                                .content_type(ContentType::json())
                                .json(response);
                        } 
                        /* 
                            * Caso a query tenha sido executada corretamente e 
                            * nenhum usuario foi afetado com a atualização
                        */ else {
                            
                            let error_json = UserDeleteError {
                                message: "User was not deleted!", 
                                error: "Query gone fine but user may not exists in our database."
                            };
        
                            // Retornar com status 404
                            return HttpResponse::NotFound()
                                .content_type(ContentType::json())
                                .json(error_json);
                        }
                    },
                    Err(error) => {
                        // Resposta com o erro que aconteceu no lado do servidor
                        let error_json = UserDeleteError {
                            message: "Some error raised deleting user", 
                            error: &error.to_string()
                        };
        
                        // Retornar resposta com status 500
                        return HttpResponse::InternalServerError()
                            .content_type(ContentType::json())
                            .json(error_json);
                    }
                };
            },
            
            // Em caso de erro, geralmente o token JWT não é valido.
            Err(_error) => {

                let error_json = AuthLoginError {
                    message: "Cannot decode JWT!",
                    error: "Your auth value may not be a valid Json Web Token."
                };

                return HttpResponse::Unauthorized()
                    .content_type(ContentType::json())
                    .json(error_json);
                
            }
        }
    } else {
        let error_json = GenericError {
            message: "Error trying delete account!",
            error: "No JWT Token was provided."
        };

        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .json(error_json);
    }
}

pub async fn activate_user(
    path: web::Path<String>, 
    body: web::Json<UserActivateRequest>
) -> impl Responder {
    let validate = body.validate();
    
    match validate {
        Ok(_) => {

                let data = body.into_inner();
                let conn = &mut get_connection().await.unwrap();
                let email_user = data.email;

                let mut found_user = users::table
                    .filter(users::email.eq(email_user))
                    .select(User::as_select())
                    .get_result::<User>(conn)
                    .await
                    .expect("User not found! Query failed!");

                let key_value = format!("user:{}:uuid_user_activate", found_user.id);
            
                if path.into_inner() == cache_get_key::<&str, String>(&key_value).await.unwrap() {
                    let new_password_hashed = match bcrypt::hash(
                            &data.new_password, 
                            BCRYPT_COST.to_owned()
                        ) {
                        Ok(password_data) => password_data,
                        Err(_err) => panic!("Error while bcrypt password")
                    };
                    
                    found_user.password = Some(new_password_hashed);
                    found_user.updated_at = Some(Utc::now());

                    let update_query = conn.transaction(
                        |con| async move {
                        
                        let update_user_password = diesel::update(users::table
                            .filter(users::id.eq(found_user.id))
                            ).set(found_user)
                            .get_result::<User>(con)
                            .await;


                        return update_user_password;
                        
                    }.scope_boxed()).await;
                    
                    cache_del_key::<String, String>(key_value).await.unwrap();

                    match update_query {
                        Ok(_user) => {
                            let user_activated_response = GenericResponse {
                                message: "User activated successfully!"
                            };

                            return HttpResponse::Ok()
                            .content_type(ContentType::json())
                            .json(user_activated_response);
                        },
                        Err(_) => {
                            let query_error_response = GenericError {
                                message: "Internal server error while activating user!",
                                error: "Error in activate user query in database."
                            };

                            return HttpResponse::InternalServerError()
                            .content_type(ContentType::json())
                            .json(query_error_response);
                        }
                    }
                } else {
                    let uuid_not_exists_err = UserUpdateError {
                        message: "Bad Request while activating user!",
                        error: "UUID do not exists."
                    };

                    return HttpResponse::BadRequest().content_type(ContentType::json()).json(uuid_not_exists_err);
                }
            }, Err(error) => {
            return HttpResponse::BadRequest().content_type(ContentType::json()).json(error);
        }
    }
}

async fn resend_user_activation_hash(body: web::Json<UserResendActivationHashRequest>) -> impl Responder {

    let validate = body.validate();
    
    match validate {
        Ok(_) => {
            let data = body.into_inner();
            let conn = &mut get_connection().await.unwrap();
            
            let found_user = users::table
                .filter(users::email.eq(data.email))
                .select(User::as_select())
                .get_result::<User>(conn)
                .await;

            match found_user {
                Ok(user) => {

                    let context = ContextV7::new();
                    let time_unix = Timestamp::now(&context).to_unix();

                    let uuid_user = Uuid::new_v7(
                        Timestamp::from_unix(&context,time_unix.0, time_unix.1)
                    );

                    let cache_uuid = cache_set_key::<&str, String, ()>(
                        &format!("user:{}:uuid_user_activate", &user.id),
                        uuid_user.to_string(),
                        86400
                    ).await;

                    match cache_uuid {
                        Ok(_) => {
                            let message_email = format!("This is your Token for define your password {}", uuid_user.to_string());

                            let email_text_body = SinglePart::builder()
                                .header(header::ContentType::TEXT_PLAIN)
                                .body(message_email);

                            // Ler dados do usuário da aplicação (.env) e de quem vai receber o email
                            let google_email = dotenv!("EMAIL");
                            let user_receiver = user.email;
                            let google_token = dotenv!("GOOGLE_TOKEN");
                            
                            // Criar o Email
                            let email_singlepart = Message::builder()
                                .from(google_email.parse().unwrap())
                                .to(user_receiver.parse().unwrap())
                                .subject("Ativar conta Logistica-APP")
                                .singlepart(email_text_body).unwrap();

                            // Resgatar as credenciais para conexão segura
                            let creds = Credentials::new(google_email.to_owned(), google_token.to_owned());

                            // Construtor do algoritmo de transporte pelo serviço do Gmail
                            let mailer = SmtpTransport::starttls_relay("smtp.gmail.com").expect("Error creating StartTLS Transport")
                                .authentication(vec![Mechanism::Plain])
                                .credentials(creds)
                                .build();

                            let response = GenericResponse {
                                message: "A new hash sent to your email!"
                            };

                            match mailer.send(&email_singlepart) {
                                Ok(_) => return HttpResponse::Ok()
                                    .content_type(ContentType::json())
                                    .json(response),
                                Err(e) => return HttpResponse::BadGateway()
                                    .content_type(ContentType::json())
                                    .json(EmailSendError {
                                        message: "Error sending email!",
                                        error: &e.to_string()
                                    }),
                            }
                        },
                        Err(error) => {
                            let error_msg = error.to_string();

                            let response = GenericError {
                                message: "Error while setting uuid key in cache!",
                                error: &error_msg
                            };
                            
                            return HttpResponse::InternalServerError()
                                .content_type(ContentType::json())
                                .json(response);
                        }
                    }
                } Err(_error) => {
                    let user_not_found_res = GenericError {
                        message: "User Not Found!",
                        error: "User with this email not exists in our App."
                    };

                    return HttpResponse::NotFound()
                        .content_type(ContentType::json())
                        .json(user_not_found_res);
                }
            }
        },
        Err(error) => {
            return HttpResponse::BadRequest().content_type(ContentType::json()).json(error);
        }
    }
}

/// Endpoint para solicitar ativação da Autenticação de dois fatores
pub async fn enable_2fa(req: HttpRequest) -> impl Responder {

    // Pegar valor do token passado no header
    let token = req.headers().get("authorization").unwrap();

    match decode_jwt(token.to_str().expect("Error casting headervalue to &str")) {
        Ok(claim) => {
            
            let conn = &mut get_connection().await.unwrap();

            let user = users::table
                .filter(users::email.eq(claim.sub))
                .select(User::as_select())
                .get_result::<User>(conn)
                .await;

            match user {
                Ok(user)=> {

                    // Gerar array de com 32 bytes aleatórios
                    let mut rng = rand::rng();
                    let mut random_bytes = [0u8; 32];
                    rng.fill(&mut random_bytes);
        
                    // Pegar nome da API no .env
                    let app_name = dotenv!("APP_NAME");
                
                    // Gerar codigo base64 com os bytes aleatórios e codificar para base 32 (revisar)
                    let random_code = BASE64_STANDARD_NO_PAD.encode(&random_bytes);
                    let base_32_code = encode(Alphabet::Rfc4648 { padding: true }, random_code.as_bytes());
        
                    // Gerar uma instância de One Timed Password
                    let totp = TOTP::new(
                        Algorithm::SHA512,
                        6,
                        1,
                        30,
                        Secret::Encoded(base_32_code).to_bytes().unwrap(),
                        Some(app_name.to_string()),
                        user.email.clone()
                    ).unwrap();
        
                    // Pegar o código de QRCode e a chave de configuração da instância
                    let qrcode_base64 = totp.get_qr_base64().unwrap();
                    let setup_key = totp.get_secret_base32();

                    // Pegar a hora do tempo universal coodernado
                    let date_now = Utc::now();
        
                    /* 
                        * Atualizar usuario no banco com a chave de configuração para ser 
                        * confirmado no endpoint de ativação do 2FA 
                    */
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
        
                    // Atualizar usuario no banco com a chave de configuração do 2FA
                    let user_updated_query = diesel::update(users::table.filter(users::id.eq(user.id)))
                        .set(new_updated_user)
                        .execute(conn)
                        .await;

                    // Verificar resultado da query
                    match user_updated_query {
                        Ok(rows) => {
                            if rows > 0 {
                                let response = UserEnable2FAResponse {
                                    message: "QR Code and Config Key generated! Confirm code at /user/activate-2fa",
                                    qrcode: &qrcode_base64,
                                    config_code: &setup_key
                                }; 
                                
                                return HttpResponse::Ok().content_type(ContentType::json()).json(response);
                            } else {
                                let res_err = UserUpdateError {
                                    message: "Error trying setting 2FA code for user!",
                                    error: "Query gone fine but 2FA code was not set for user."
                                };
                    
                                return HttpResponse::NotFound()
                                    .content_type(ContentType::json())
                                    .json(res_err);
                            }
                        },

                        // Em caso de erro interno ao executar a Query
                        Err(_err) => {
                            let res_err = UserUpdateError {
                                message: "Error trying setting 2FA code for user!",
                                error: "Internal Server error while trying set 2FA code for user."
                            };
                
                            return HttpResponse::Ok()
                                .content_type(ContentType::json())
                                .json(res_err);
                        }
                    }
                    
                },
                Err(_error)=>{

                    let res_err = UserUpdateError {
                        message: "Error trying setting 2FA code for user!",
                        error: "User not found."
                    };
        
                    return HttpResponse::NotFound()
                        .content_type(ContentType::json())
                        .json(res_err);
                }
            }

        },
        Err(_err) => {
            let error_response = GenericError {
                message: "No user Logged!",
                error: "Invalid authorization token."
            };

            HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .json(error_response)
        }
    }

} 

/// Endpoint para o usuário ativar a Autenticação de dois fatores com o
/// código aleatório do aplicativo de autenticação que o usuário utiliza.
/// 
/// Para que a instância do One Timed Password valide o código passado na requisição,
/// é necessário que o dispositivo do usuário esteja sincronizado com o Horário Universal Coordenado
pub async fn activate_2fa(req: HttpRequest, body: web::Json<UserActivate2FARequest>) -> impl Responder {
    let validate = body.validate();

    match validate {
        Ok(_) => {
            // Pegar token do header na requisição
            let token = req.headers().get("authorization").unwrap();

            match decode_jwt(token.to_str().expect("Error casting headervalue to &str")) {
                Ok(claim) => {
                    
                    let conn = &mut get_connection().await.unwrap();
        
                    // Consultar usuario no banco
                    let user = users::table
                        .filter(users::email.eq(claim.sub))
                        .select(User::as_select())
                        .get_result::<User>(conn)
                        .await;
                    
                    match user {
                        Ok(user_found) => {
                            
                            // Verificar se o usuario encontrado possui a chave de configuração do 2FA
                            match user_found.two_factor_secret {
                                Some(secret) => {
                                    
                                    // Pegar o horário universal coordenado e o nome do app (.env)
                                    let date_now = Utc::now();
                                    let app_name = dotenv!("APP_NAME");
        
                                    // Criar a instância do One Timed Password com as configurações do usuario
                                    let totp = TOTP::new(
                                        Algorithm::SHA512,
                                        6,
                                        1,
                                        30,
                                        Secret::Encoded(secret.clone()).to_bytes().unwrap(),
                                        Some(app_name.to_string()),
                                        user_found.email.clone()
                                    ).unwrap();
        
                                    // Aproximar o valor da data/hora para o horario Unix (segundos desde 01/01/1970)
                                    let seconds_now = ((Utc::now().timestamp_millis()) / 1000) as u64;
        
                                    /* 
                                        * Verificar se o código passado na requisição é valido baseado com 
                                        * a aproximação feita pro horário Unix
                                    */
                                    if totp.check(body.code.as_str(), seconds_now) == true {
                                        
                                        // Preparar Struct com o 2FA confirmado (data/hora atual UTC)
                                        let new_updated_user = UserDTO {
                                            name: user_found.name,
                                            email: user_found.email,
                                            password: user_found.password,
                                            two_factor_secret:  Some(secret),
                                            two_factor_recovery_code: user_found.two_factor_recovery_code,
                                            two_factor_confirmed_at: Some(date_now),
                                            created_at: user_found.created_at,
                                            updated_at: Some(date_now),
                                            deleted_at: user_found.deleted_at
                                        };
        
                                        // Query para atualizar usuario com o 2FA confirmado
                                        let user_updated_2fa_on = diesel::update(users::table.filter(users::id.eq(user_found.id)))
                                            .set(new_updated_user)
                                            .execute(conn)
                                            .await;
        
                                        // Verificar se a alteração foi realizada no banco
                                        match user_updated_2fa_on {
                                            Ok(rows) => {
                                                if rows > 0 {
                                                    
                                                    // Sucesso, 2FA configurado para o usuario
                                                    let response = GenericResponse {
                                                        message: "2FA setted up successfully!"
                                                    }; 
                                                    
                                                    return HttpResponse::Ok().content_type(ContentType::json()).json(response);
                                        
                                                } else {
        
                                                    // Caso a query tenha rodado mas o usuario não foi alterado no banco
                                                    let response = GenericError {
                                                        message: "Error setting up 2FA!",
                                                        error: "Query gone fine but 2FA Challenge was not confirmed on our side"
                                                    }; 
                                        
                                                    return HttpResponse::NotFound().content_type(ContentType::json()).json(response);   
        
                                                }
        
                                            },
                                            Err(_) => {
                                                let response = GenericError {
                                                    message: "Error setting up 2FA!",
                                                    error: "Internal Server error querying to DB"
                                                }; 
                                    
                                                return HttpResponse::InternalServerError().content_type(ContentType::json()).json(response);
                                            }
                                        }
                                    } else {
                                        let response = GenericError {
                                            message: "Error setting up 2FA!",
                                            error: "Invalid code, user failed 2FA Challenge!"
                                        }; 
        
                                        return HttpResponse::Unauthorized()
                                            .content_type(ContentType::json())
                                            .json(response);
                                    }
        
                                },
                                None => {
        
                                    /* 
                                        * Se o usuario não tem a chave configurada, 
                                        * foi porque ele não solicitou o 2FA para a sua conta ainda
                                    */
                                    let res_2fa_not_requested = GenericError {
                                        message: "Error trying confirm 2FA code for user!",
                                        error: "User did not request 2FA challenge!"
                                    };
        
                                    return HttpResponse::Unauthorized()
                                        .content_type(ContentType::json())
                                        .json(res_2fa_not_requested);
                                }
                        }
                    }, Err(_err) => {
        
                        // Caso as credenciais carregadas no token forem inválidas
                        let res_2fa_not_requested = GenericError {
                            message: "No user Logged!",
                            error: "Your token claims are not valid."
                        };
        
                        return HttpResponse::Unauthorized()
                            .content_type(ContentType::json())
                            .json(res_2fa_not_requested);
                    }
                }
            },
            Err(_err) => {
                    
                    // Caso o token for inválido
                    let error_response = GenericError {
                        message: "No user Logged!",
                        error: "Invalid authorization token."
                    };
        
                    return HttpResponse::Unauthorized()
                        .content_type(ContentType::json())
                        .json(error_response);
                }
            }
        },
        Err(error) => {
            return HttpResponse::BadRequest().content_type(ContentType::json()).json(error);
        }
    }
    
}

/// Endpoints de usuários
pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
web::scope("/users")
            .route("/resend-activation-hash", web::post().to(resend_user_activation_hash))
            .route("/activate-user/{uuid}", web::put().to(activate_user))
            .service(web::scope("")
                .route("/deleteMyAccount",web::delete().to(delete_my_account))
                .route("/update/{id}", web::put().to(update))
                .route("/index", web::get().to(index))
                .route("/enable-2fa", web::get().to(enable_2fa))
                .route("/activate-2fa", web::post().to(activate_2fa))
                .route("/store", web::post().to(store))
                .wrap(from_fn(auth_middleware))
            )
    );

}