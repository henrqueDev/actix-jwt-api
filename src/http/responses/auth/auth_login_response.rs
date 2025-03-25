use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthLoginResponse<'a> {
    pub message: &'a str,
    pub token: Option<&'a str>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthLoginError<'a> {
    pub message: &'a str
}