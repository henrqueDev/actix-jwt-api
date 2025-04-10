use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct UserResendActivationHashRequest {
    #[validate(email)]
    pub email: String
}