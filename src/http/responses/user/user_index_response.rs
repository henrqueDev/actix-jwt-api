use serde::{Deserialize, Serialize};

use crate::model::user::user_dto::UserDTOMin;
#[derive(Debug, Serialize, Deserialize)]
pub struct UserIndexResponse<'a> {
    pub message: &'a str,
    pub users: Vec<UserDTOMin>,
    pub current_page: Option<u32>,
    pub per_page: Option<u32>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserIndexError<'a> {
    pub message: &'a str,
    pub error: &'a str
}