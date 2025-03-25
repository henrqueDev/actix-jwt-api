use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthLoginRequest {
    pub email: String,
    pub password: String,
    pub code: Option<String>
}