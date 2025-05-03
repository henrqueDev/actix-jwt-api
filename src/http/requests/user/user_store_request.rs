use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UserStoreRequest {
    pub name: String,
    #[validate(email(message = "Invalid email! Insert a valid email."))]
    pub email: String
}