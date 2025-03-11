use actix_multipart::form::{bytes::Bytes, text::Text, MultipartForm};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailSendRequest {
    pub title: String,
    pub content: String,
    pub to: String
}

#[derive(Debug, MultipartForm)]
pub struct EmailSendRequestFormData {
    #[multipart(limit = "10MiB")]
    pub files: Vec<Bytes>,
    pub title: Text<String>,
    pub content: Text<String>,
    pub to: Text<String>
}
