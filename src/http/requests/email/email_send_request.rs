use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailSendRequest {
    pub title: String,
    pub content: String,
    pub to: String
}