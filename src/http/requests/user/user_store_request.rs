use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserStoreRequest {
    name: String,
    emai: String,
    password: String
}