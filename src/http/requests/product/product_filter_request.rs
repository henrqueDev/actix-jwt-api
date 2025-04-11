use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ProductFilterRequest {
    pub id: Option<i32>,
    pub sku: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f32>,
    pub weight: Option<f32>,
    pub dimension_height: Option<f32>,
    pub dimension_width: Option<f32>,
    pub dimension_depth: Option<f32>,
    pub product_category: Option<String>,
    pub per_page: Option<u32>,
    pub page: Option<u32>
}