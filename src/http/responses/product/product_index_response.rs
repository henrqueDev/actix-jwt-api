use serde::{Deserialize, Serialize};

use crate::model::product::product::Product;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductIndexResponse<'a> {
    pub message: &'a str,
    pub products: Vec<Product>,
    pub current_page: Option<u32>,
    pub per_page: Option<u32>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductIndexError<'a> {
    pub message: &'a str,
    pub error: &'a str
}