use serde::{Deserialize, Serialize};
use crate::models::user::user::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStoreResponse<'a> {
    pub message: &'a str,
    pub user: User
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStoreError<'a> {
    pub message: &'a str,
    pub error: &'a str
}