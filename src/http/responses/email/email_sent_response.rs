use serde::Serialize;
use crate::http::requests::email::email_send_request::EmailSendRequest;

#[derive(Serialize)]
pub struct EmailSentResponse<'a>{
    pub message: &'a str,
    pub email: &'a EmailSendRequest
}

#[derive(Serialize)]
pub struct EmailSendError<'a> {
    pub message: &'a str,
    pub error: &'a str
}