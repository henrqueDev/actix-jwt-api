use serde::{Deserialize, Serialize};
use crate::model::product::product::Product;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductStoreResponse<'a> {
    pub message: &'a str,
    pub product: Product
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductStoreError<'a> {
    pub message: &'a str,
    pub error: &'a str
}