use serde::{Deserialize, Serialize};
use crate::models::user::user::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserUpdateResponse<'a> {
    pub message: &'a str,
    pub user: User
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserUpdateError<'a> {
    pub message: &'a str,
    pub error: &'a str
}