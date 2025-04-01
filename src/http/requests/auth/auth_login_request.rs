use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct AuthLoginRequest {
    pub email: String,
    pub password: String,
    #[validate(length(equal=6, message="2FA code must be equal 6 digits!"))]
    pub code: Option<String>
}