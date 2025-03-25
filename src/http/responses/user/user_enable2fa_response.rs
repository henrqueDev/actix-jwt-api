use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserEnable2FAResponse<'a> {
    pub message: &'a str,
    pub qrcode: &'a str,
    pub config_code: &'a str
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserEnable2FAError<'a> {
    pub message: &'a str,
    pub error: &'a str
}