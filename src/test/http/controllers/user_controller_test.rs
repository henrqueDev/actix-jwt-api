#[cfg(test)]
mod user_controller_test {

    use actix_web::{body::to_bytes, http::header::{self, ContentType, HeaderValue, TryIntoHeaderPair}, test, App};
    use crate::http::{
        controllers::auth_controller,
        requests::{auth::auth_login_request::AuthLoginRequest, user::user_store_request::UserStoreRequest},
        responses::auth::auth_login_response::AuthLoginResponse,
    };

    #[actix_web::test]
    async fn test_auth_login() {
        let app = test::init_service(App::new()
            .configure(auth_controller::config)
        ).await;

        let user_login = AuthLoginRequest {
            email: Some("cleverton@example.com".to_owned()),
            password: Some("cleverton123".to_owned()),
            code: None
        };

        let req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(user_login)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status().is_success();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8(body_bytes.clone().to_vec()).unwrap();
        
        let _parsed: AuthLoginResponse = serde_json::from_str(&body_str).unwrap();
        
        assert!(status);
    }

    #[actix_web::test]
    async fn test_store_user() {
        let app = test::init_service(App::new()
            .configure(auth_controller::config)
        ).await;

        let user_login = AuthLoginRequest {
            email: Some("cleverton@example.com".to_owned()),
            password: Some("cleverton123".to_owned()),
            code: None
        };

        let req = test::TestRequest::post().set_json(user_login).uri("/auth/login").to_request();
        let resp = test::call_service(&app, req).await;
        let status_login = resp.status().is_success();

        
        let bytes = to_bytes(resp.into_body()).await.unwrap();
        let bytes_string = String::from_utf8(bytes.to_vec()).unwrap();
        let json: AuthLoginResponse = serde_json::from_str(&bytes_string).unwrap();
        assert!(status_login); 

        let auth_header = TryIntoHeaderPair::try_into_pair(
            (header::AUTHORIZATION, HeaderValue::from_str(json.token.unwrap()).unwrap())).unwrap();
    
        let email_test = "testeshenrique12345@gmail.com";
        
        let user_store_data = UserStoreRequest {
            name: "Fulano_teste".to_string(),
            email: email_test.to_string()
        };
        
        let req_store_user = test::TestRequest::post()
            .insert_header(ContentType::json())
            .insert_header(auth_header.clone())
            .set_json(user_store_data)
            .uri("/users/store")
            .to_request();

        let resp_store_user= test::call_service(&app, req_store_user).await;

        if resp_store_user.status().is_success() {
            assert!(resp_store_user.status().is_success());
        } else if resp_store_user.status().as_u16() == 409 {
            
            let user_store_data = UserStoreRequest {
                name: "Fulano_teste".to_string(),
                email: email_test.to_string()
            };
            
            let req_store_user = test::TestRequest::post()
                .insert_header(ContentType::json())
                .insert_header(auth_header)
                .set_json(user_store_data)
                .uri("/users/resend-activation-hash")
                .to_request();

            let resp_resend_activation_email= test::call_service(&app, req_store_user).await;

            assert!(resp_resend_activation_email.status().is_success());
        }
    }
}