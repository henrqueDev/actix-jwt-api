use serde::Deserialize;
use validator::Validate;


#[derive(Debug, Deserialize, Validate)]
pub struct ProductStoreRequest {
    #[validate(length(min = 3, message = "SKU must be greater than 3 chars!"))]
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price: f32,
    pub weight: f32,
    pub dimension_height: f32,
    pub dimension_width: f32,
    pub dimension_depth: f32,
    pub product_category_id: i32
}