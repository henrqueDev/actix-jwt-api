use serde::{Deserialize, Serialize};
use crate::model::user::user::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStoreResponse {
    pub message: String,
    pub user: User
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStoreError {
    pub message: String,
    pub error: String
}