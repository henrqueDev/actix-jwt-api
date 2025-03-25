use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDeleteResponse<'a> {
    pub message: &'a str,
    pub email: &'a str
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDeleteError<'a> {
    pub message: &'a str,
    pub error: &'a str
}