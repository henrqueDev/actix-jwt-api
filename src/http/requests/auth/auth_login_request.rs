use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct AuthLoginRequest {
    #[validate(required)]
    pub email: Option<String>,
    #[validate(required)]
    pub password: Option<String>,
    #[validate(length(equal=6, message="2FA code must be equal 6 digits!"))]
    pub code: Option<String>
}