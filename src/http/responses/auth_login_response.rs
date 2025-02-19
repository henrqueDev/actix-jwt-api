use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthLoginResponse {
    pub message: String,
    pub token: Option<String>
}