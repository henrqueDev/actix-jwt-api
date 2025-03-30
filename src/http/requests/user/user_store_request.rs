use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct UserStoreRequest {
    pub name: String,
    #[validate(email(message = "Invalid email! Insert a valid email."))]
    pub email: String,
    pub password: String
}