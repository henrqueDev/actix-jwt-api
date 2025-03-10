use actix_multipart::form::MultipartForm;
use actix_web::{http::header::ContentType, middleware::from_fn, web::{self, ServiceConfig}, HttpResponse, Responder};
use lettre::{message::{Attachment, MultiPart, SinglePart}, transport::smtp::authentication::{Credentials, Mechanism}, Message, SmtpTransport, Transport};
use crate::http::{middleware::auth_middleware::auth_middleware, requests::email::email_send_request::{EmailSendRequest, EmailSendRequestFormData}, responses::email::email_sent_response::{EmailSendError, EmailSentResponse}};
use dotenv_codegen::dotenv;
use lettre::message::header;
use lettre::message::header::ContentType as EmailContentType;

pub async fn send(body: MultipartForm<EmailSendRequestFormData>) -> impl Responder {

    let data = &body;
    let filename = &data.file.file_name.clone().unwrap_or_else(|| "anexo".to_owned());
    let file_bytes = &data.file.data.to_vec();
    
    let user_email = dotenv!("EMAIL");
    let user_receiver = &data.to;
    let password = dotenv!("GOOGLE_TOKEN");

    let content_type = EmailContentType::parse("application/octet-stream").unwrap();
    let attachment = Attachment::new(filename.to_owned())
        .body(file_bytes.to_owned(), content_type);
    
    let text_body = SinglePart::builder().header(header::ContentType::TEXT_PLAIN).body(data.content.clone());
    let multipart_body = MultiPart::mixed().singlepart(attachment).singlepart(text_body);
    
    let email = Message::builder()
        .from(user_email.parse().unwrap())
        .to(user_receiver.parse().unwrap())
        .subject(data.title.clone())
        .multipart(multipart_body).unwrap();

    let creds = Credentials::new(user_email.to_owned(), password.to_owned());

    let mailer = SmtpTransport::starttls_relay("smtp.gmail.com").expect("Error creating StartTLS Transport")
        .authentication(vec![Mechanism::Plain])
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(EmailSentResponse {
                message: "Email sent successfully!",
                email: &data
            }),
        Err(e) => HttpResponse::BadGateway()
            .content_type(ContentType::json())
            .json(EmailSendError {
                message: "Error sending email!",
                error: &e.to_string()
            }),
    }

}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
web::scope("/email")
            .route("/send", web::post().to(send))
            .wrap(from_fn(auth_middleware))
    );
}