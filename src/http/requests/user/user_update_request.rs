use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize)]
pub struct UserUpdateRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub new_password: Option<String>,
    pub old_password: Option<String>
}