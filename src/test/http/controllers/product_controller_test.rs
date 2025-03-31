#[cfg(test)]
mod product_controller_test {
   
    use actix_web::{test, App};

    use crate::http::controllers::product_controller;

    #[actix_web::test]
    async fn test_index_get() {
        let app = test::init_service(App::new()
            .configure(product_controller::config)
        ).await;

        let req = test::TestRequest::get().uri("/products/index").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}