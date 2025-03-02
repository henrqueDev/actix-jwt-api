use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDeleteResponse {
    pub message: String,
    pub email: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDeleteError {
    pub message: String,
    pub error: String
}