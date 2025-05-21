#[cfg(test)]
mod product_controller_test {
   
    use actix_web::{body::to_bytes, http::header::{self, ContentType, HeaderValue, TryIntoHeaderPair}, test, App};

    use crate::http::{controllers::{auth_controller, product_controller}, requests::auth::auth_login_request::AuthLoginRequest, responses::auth::auth_login_response::AuthLoginResponse};

    #[actix_web::test]
    async fn test_product_index_get() {
        let app = test::init_service(App::new()
            .configure(auth_controller::config)
            .configure(product_controller::config)
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

        let req_products = test::TestRequest::get()
            .insert_header(ContentType::json())
            .insert_header(auth_header)
            .uri("/products/index")
            .to_request();
        let resp_products = test::call_service(&app, req_products).await;

        assert!(resp_products.status().is_success());
    }
}