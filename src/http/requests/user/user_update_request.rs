use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate)]
pub struct UserUpdateRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    #[validate(must_match(other = "new_password", message="New password confirmation failed!"))]
    pub confirm_new_password: Option<String>,
    #[validate(must_match(other = "confirm_new_password", message="New password confirmation failed!"))]
    pub new_password: Option<String>,
    pub old_password: Option<String>
}