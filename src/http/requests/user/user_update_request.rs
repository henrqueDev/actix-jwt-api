use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize)]
pub struct UserUpdateRequest {
    pub name: String,
    pub email: String,
    pub new_password: String,
    pub old_password: String
}