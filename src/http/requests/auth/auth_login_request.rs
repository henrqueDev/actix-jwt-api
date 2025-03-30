use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct AuthLoginRequest {
    pub email: String,
    pub password: String,
    #[validate(length(equal=6, message="2FA code must be equal 6 digits!"))]
    pub code: Option<String>
}