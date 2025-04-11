use actix_web::{http::header::ContentType, middleware::from_fn, web::{self, ServiceConfig}, HttpResponse, Responder};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, TextExpressionMethods};
use diesel_async::RunQueryDsl;
use validator::Validate;
use crate::{database::db::get_connection, http::{middleware::auth_middleware::auth_middleware, requests::product::{product_filter_request::ProductFilterRequest, product_store_request::ProductStoreRequest, product_update_request::ProductUpdateRequest}, responses::product::{product_store_response::{ProductStoreError, ProductStoreResponse}, product_update_response::ProductUpdateResponse}, GenericError}, model::product::{product::Product, product_dto::ProductDTO}, schema::{product_categories, products}};
use crate::http::responses::product::product_index_response::ProductIndexResponse;

/// Endpoint para consulta de products com filtros opcionais
async fn index(query_params: web::Query<ProductFilterRequest>) -> impl Responder {

    let conn = &mut get_connection().await.unwrap();

    let mut query = products::table.into_boxed();

    // Aplicando filtros na consulta (Revisar o framework)
    if let Some(id_query) = query_params.0.id {
        query = query.filter(products::id.eq(id_query));
    }

    if let Some(name_query) = query_params.0.name {
        query = query.filter(products::name.like(format!("%{}%", name_query)));
    }

    if let Some(sku_query) = query_params.0.sku {
        query = query.filter(products::sku.eq(sku_query));
    }

    if let Some(description_query) = query_params.0.description {
        query = query.filter(products::description.like(format!("%{}%", description_query)));
    }

    if let Some(dimension_width_query) = query_params.0.dimension_width {
        query = query.filter(products::dimension_width.ge(dimension_width_query));
    }

    if let Some(product_category) = query_params.0.product_category {
        let query_category_id: Result<i32, _> = product_categories::table
            .filter(product_categories::name.eq(product_category))
            .select(product_categories::id)
            .get_result::<i32>(conn)
            .await;

        match query_category_id {
            Ok(id) => {
                query = query.filter(products::product_category_id.eq(id));
            },
            Err(_) => {
                let bad_request_cat_res = GenericError {
                    message: "Bad Request querying Products!",
                    error: "Product Category do not exists."
                };

                return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .json(bad_request_cat_res);
            }
        };

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

    // Apenas listar produtos não excluidos
    query = query.filter(products::deleted_at.is_null());

    // Consulta na tabela de products, retornando a struct Product
    let results = query
        .select(Product::as_select())
        .get_results::<Product>(conn)
        .await;

    // Verificar se a consulta foi um sucesso (revisar tipagem do retorno de erro)
    match results {
        Ok(query_products) => {

            // Preparando dados para retornar para o client
            let products_response = ProductIndexResponse {
                message: "Query products gone successfully!",
                products: query_products,
                current_page: query_params.0.page,
                per_page: query_params.0.per_page
            };

            // Resposta status 200
            return HttpResponse::Ok().content_type(ContentType::json()).json(products_response);
        },
        Err(_err) => {

            // Erro interno no servidor durante consulta no banco
            let error_response = GenericError {
                message: "Error querying products on DB!",
                error: "Internal Server error while querying products."
            };

            // Resposta com status 500
            return HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .json(error_response);
        }
    }
}

/// Endpoint para atualizar produto
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
                            
                            let success_response = ProductUpdateResponse {
                                message: "Product stored successfully!",
                                product: product_updated
                            };

                            return HttpResponse::Ok().content_type(ContentType::json()).json(success_response);
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

/// Endpoint para cadastrar novo Produto
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

                    match store_product {
                        Ok(product_stored) => {
                            let res_success =  ProductStoreResponse {
                                message: "Product stored successfully!",
                                product: product_stored
                            };
                            
                            return HttpResponse::Ok()
                                .content_type(ContentType::json())
                                .json(res_success);
                        },
                        Err(_) => {
                            let res_conflict = ProductStoreError {
                                message: "Error storing product!",
                                error: "Product with same SKU already exists!"
                            };

                            return HttpResponse::Conflict()
                                .content_type(ContentType::json())
                                .json(res_conflict);
                        }
                    }
                
                    
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

/// Endpoint para Excluir produto (Exclusão lógica)
async fn delete(path: web::Path<u32>) -> impl Responder{
    let id = path.into_inner();
            
    let conn = &mut get_connection().await.unwrap();
    let product = products::table
        .filter(products::id.eq(id as i32))
        .select(Product::as_select())
        .get_result::<Product>(conn)
        .await;

    match product {
        Ok(product) => {
            match &product.deleted_at {
                Some(_) => {
                    let err_not_found = GenericError {
                        message: "Error deleting product!",
                        error: "Product was already deleted."
                    };
        
                    return HttpResponse::Conflict().content_type(ContentType::json()).json(err_not_found);        
                },
                None => {
                    let mut product_to_delete = ProductDTO {
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
        
                    let time_now = Utc::now();
        
                    // Exclusão Lógica
                    product_to_delete.deleted_at = Some(time_now);
        
                    let delete_product = diesel::update(products::table.filter(products::id.eq(id as i32)))
                        .set(product_to_delete).get_result::<Product>(conn).await;
        
                    match delete_product {
                        Ok(product_deleted) => {
                            let success_restore_product = ProductUpdateResponse {
                                message: "Product deleted successfully!",
                                product: product_deleted
                            };
        
                            return HttpResponse::Ok().content_type(ContentType::json()).json(success_restore_product);
                        },
                        Err(_) => {
                            let err_not_found = GenericError {
                                message: "Error deleting product!",
                                error: "Internal server error while deleting product!"
                            };
        
                            return HttpResponse::InternalServerError().content_type(ContentType::json()).json(err_not_found);
                        }
                    }
                }
            }

        }, Err(_error) => {
            let err_not_found = GenericError {
                message: "Error deleting product!",
                error: "Product was not found."
            };

            return HttpResponse::NotFound().content_type(ContentType::json()).json(err_not_found);            
        }
    }
}

/// Endpoint Restaurar produto excluido (Exclusão lógica)
async fn restore(path: web::Path<u32>) -> impl Responder{
    let id = path.into_inner();
            
    let conn = &mut get_connection().await.unwrap();
    let product = products::table
        .filter(products::id.eq(id as i32))
        .select(Product::as_select())
        .get_result::<Product>(conn)
        .await;

    match product {
        Ok(product) => {
            let mut product_to_restore = ProductDTO {
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

            // Restaurar produto
            product_to_restore.deleted_at = None;

            let product_restore = diesel::update(products::table.filter(products::id.eq(id as i32)))
                .set(product_to_restore).get_result::<Product>(conn).await;

            match product_restore {
                Ok(product_restored) => {
                    let success_restore_product = ProductUpdateResponse {
                        message: "Product restored successfully!",
                        product: product_restored
                    };

                    return HttpResponse::Ok().content_type(ContentType::json()).json(success_restore_product);
                },
                Err(_) => {
                    let err_not_found = GenericError {
                        message: "Error restoring product!",
                        error: "Internal server error while restoring product!"
                    };

                    return HttpResponse::InternalServerError().content_type(ContentType::json()).json(err_not_found);
                }
            }


        }, Err(_error) => {
            let err_not_found = GenericError {
                message: "Error restoring product!",
                error: "Product was not found."
            };

            return HttpResponse::NotFound().content_type(ContentType::json()).json(err_not_found);
        }
    }
}

/// Rotas de Produtos
pub fn config(cfg: &mut ServiceConfig) -> () {
    cfg.service(
    web::scope("/products")
                .route("/update/{id}", web::put().to(update))
                .route("/store", web::post().to(store))
                .route("/index", web::get().to(index))
                .route("/delete/{id}", web::delete().to(delete))
                .route("/restore/{id}", web::put().to(restore))
                .wrap(from_fn(auth_middleware))
    );
}