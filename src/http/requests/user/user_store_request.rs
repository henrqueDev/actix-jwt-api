use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserStoreRequest {
    pub name: String,
    pub email: String,
    pub password: String
}