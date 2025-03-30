use actix_web::{http::header::ContentType, middleware::from_fn, web::{self, ServiceConfig}, HttpResponse, Responder};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use validator::Validate;
use crate::{database::db::get_connection, http::{middleware::auth_middleware::auth_middleware, requests::product::{product_store_request::ProductStoreRequest, product_update_request::ProductUpdateRequest}, GenericError}, model::product::{product::Product, product_dto::ProductDTO}, schema::{product_categories, products}};

async fn update(path: web::Path<u32>,body: web::Json<ProductUpdateRequest>) -> impl Responder {
    
    let validate = body.validate();
    let id = path.into_inner();

    match validate {
        Ok(_) => {
            
            let conn = &mut get_connection().await.unwrap();
            let data = body.into_inner();

            let product = products::table
                .filter(products::id.eq(id as i32))
                .select(Product::as_select())
                .get_result::<Product>(conn)
                .await;

            match product {
                Ok(product) => {
                    let mut new_product_updated = ProductDTO {
                        sku: product.sku,
                        name: product.name,
                        description: product.description,
                        price: product.price,
                        weight: product.weight,
                        dimension_height: product.dimension_height,
                        dimension_width: product.dimension_width,
                        dimension_depth: product.dimension_depth,
                        product_category_id: product.product_category_id,
                        created_at: product.created_at,
                        updated_at: product.updated_at,
                        deleted_at: product.deleted_at,
                    };

                    // Verificar se o campo de nome foi passado na requisição
                    match &data.name {
                        Some(new_name) => {new_product_updated.name = new_name.to_owned()},
                        None => {},
                    }

                    match &data.sku {
                        Some(new_sku) => {new_product_updated.sku = new_sku.to_owned()},
                        None => {},
                    }

                    match &data.weight {
                        Some(new_weight) => {new_product_updated.weight = new_weight.to_owned()},
                        None => {}
                    }

                    match &data.description {
                        Some(new_description) => {new_product_updated.description = new_description.to_owned()},
                        None => {},
                    }

                    match &data.dimension_width {
                        Some(new_width) => {new_product_updated.dimension_width = new_width.to_owned()},
                        None => {}
                    }

                    match &data.dimension_height {
                        Some(new_height) => {new_product_updated.dimension_height = new_height.to_owned()},
                        None => {}
                    }

                    match &data.dimension_depth {
                        Some(new_depth) => {new_product_updated.dimension_depth = new_depth.to_owned()},
                        None => {}
                    }

                    match data.product_category_id {
                        Some(category_id) => {
                            
                            let category = product_categories::table
                                .filter(product_categories::id.eq(category_id))
                                .select(product_categories::id)
                                .get_result::<i32>(conn)
                                .await;

                            match category{
                                Ok(category_found_id) => {
                                    new_product_updated.product_category_id = category_found_id;                                    
                                },
                                Err(_) => {
                                    let err_not_found = GenericError {
                                        message: "Error updating Category of product!",
                                        error: "Product category was not found."
                                    };

                                    return HttpResponse::NotFound().content_type(ContentType::json()).json(err_not_found);
                                }
                            }
                        },
                        None => {},
                    }

                    let time_now = Utc::now();

                    new_product_updated.updated_at = Some(time_now);

                    let update_product = diesel::update(products::table.filter(products::id.eq(id as i32)))
                        .set(new_product_updated).get_result::<Product>(conn).await;

                    match update_product {
                        Ok(product_updated) => {
                            return HttpResponse::Ok().content_type(ContentType::json()).json(product_updated);
                        },
                        Err(_) => {
                            let err_not_found = GenericError {
                                message: "Error updating product!",
                                error: "Internal server error while updating product!"
                            };

                            return HttpResponse::InternalServerError().content_type(ContentType::json()).json(err_not_found);
                        }
                    }


                }, Err(_error) => {
                    let err_not_found = GenericError {
                        message: "Error updating product!",
                        error: "Product was not found."
                    };

                    return HttpResponse::NotFound().content_type(ContentType::json()).json(err_not_found);
                }
            }
        },
        Err(error) => {
            return HttpResponse::BadRequest().content_type(ContentType::json()).json(error)
        }
    }
}

async fn store(body: web::Json<ProductStoreRequest>) -> impl Responder {
    let validate = body.validate();

    match validate {
        Ok(_) => {
            let data = body.into_inner();
            let conn = &mut get_connection().await.unwrap();
        
            let product_cat_id = product_categories::table
                .filter(product_categories::id.eq(&data.product_category_id))
                .select(product_categories::id)
                .get_result::<i32>(conn)
                .await;
        
            match product_cat_id {
                Ok(category_id) => {
                    let time_now = Utc::now();
        
                    let product_data = ProductDTO {
                        sku: data.sku,
                        name: data.name,
                        description: data.description,
                        price: data.price,
                        weight: data.weight,
                        dimension_height: data.dimension_height,
                        dimension_width: data.dimension_width,
                        dimension_depth: data.dimension_depth,
                        product_category_id: category_id,
                        created_at: Some(time_now),
                        updated_at: Some(time_now),
                        deleted_at: None,
                    };
                
                    let store_product = diesel::insert_into(products::table)
                        .values(product_data)
                        .get_result::<Product>(conn)
                        .await;
                
                    return HttpResponse::Ok()
                        .content_type(ContentType::json())
                        .json(store_product.unwrap());
                },
                Err(_err) => {
                    let err_not_found = GenericError {
                        message: "Error creating new product!",
                        error: "Product category was not found."
                    };
        
                    return HttpResponse::NotFound()
                        .content_type(ContentType::json())
                        .json(err_not_found);
                }
            }
        }, Err(error) => {
            return HttpResponse::BadRequest()
                            .content_type(ContentType::json())
                            .json(error);
        }
    }
}

// Rotas de Produtos
pub fn config(cfg: &mut ServiceConfig) -> () {
    cfg.service(
    web::scope("/products")
                .route("/update/{id}", web::put().to(update))
                .route("/store", web::post().to(store))
                .wrap(from_fn(auth_middleware))
    );
}