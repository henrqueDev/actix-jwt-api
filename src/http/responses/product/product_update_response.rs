use serde::{Deserialize, Serialize};
use crate::models::product::product::Product;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductUpdateResponse<'a> {
    pub message: &'a str,
    pub product: Product
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductUpdateError<'a> {
    pub message: &'a str,
    pub error: &'a str
}