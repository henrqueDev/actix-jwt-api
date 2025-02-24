use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthLoginResponse {
    pub message: String,
    pub token: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthLoginError {
    pub message: String
}