use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct UserActivateRequest {
    #[validate(must_match(other = "new_password", message="New password confirmation failed!"))]
    pub confirm_new_password: String,
    #[validate(must_match(other = "confirm_new_password", message="New password confirmation failed!"))]
    pub new_password: String,
}