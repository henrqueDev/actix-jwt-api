use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserEnable2FAResponse<'a> {
    pub message: &'a str,
    pub qrcode: String,
    pub config_code: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserEnable2FAError {
    pub message: String,
    pub error: String
}