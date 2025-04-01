#[cfg(test)]
mod user_controller_test {

    use actix_web::{body::to_bytes, test, App};
    use crate::http::{
        controllers::auth_controller,
        requests::auth::auth_login_request::AuthLoginRequest,
        responses::auth::auth_login_response::AuthLoginResponse,
    };

    #[actix_web::test]
    async fn test_auth_login() {
        let app = test::init_service(App::new()
            .configure(auth_controller::config)
        ).await;

        let user_login = AuthLoginRequest {
            email: "cleverton@example.com".to_owned(),
            password: "cleverton123".to_owned(),
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
}