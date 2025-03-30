use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ProductUpdateRequest {
    #[validate(length(min = 3, message = "SKU must be greater than 3 chars!"))]
    pub sku: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f32>,
    pub weight: Option<f32>,
    pub dimension_height: Option<f32>,
    pub dimension_width: Option<f32>,
    pub dimension_depth: Option<f32>,
    pub product_category_id: Option<i32>
}