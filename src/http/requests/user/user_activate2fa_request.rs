use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserActivate2FARequest {
    pub code: String
}