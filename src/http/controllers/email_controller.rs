use actix_multipart::form::MultipartForm;
use actix_web::{http::header::ContentType, middleware::from_fn, web::{self, ServiceConfig}, HttpResponse, Responder};
use lettre::{message::{Attachment, MultiPart, SinglePart}, transport::smtp::authentication::{Credentials, Mechanism}, Message, SmtpTransport, Transport};
use crate::{http::{middleware::auth_middleware::auth_middleware, requests::email::email_send_request::{EmailSendRequest, EmailSendRequestFormData}, responses::email::email_sent_response::{EmailSendError, EmailSentResponse}, GenericError}, services::{google_oauth2::refresh_oauth2_google, redis_client::cache_get_key}};
use dotenvy_macro::dotenv;
use lettre::message::header;
use lettre::message::header::ContentType as EmailContentType;

/// Endpoint para envio de Email
pub async fn send(body: MultipartForm<EmailSendRequestFormData>) -> impl Responder {

    // Pegar referencia do formulario passado na requisição
    let data = &body;
    
    // Criar um singlePart do texto do email
    let text_body = SinglePart::builder().header(header::ContentType::TEXT_PLAIN).body(data.content.clone());
    
    // Iniciar a construção do Multipart
    let mut multipart_body = MultiPart::mixed().singlepart(text_body);

    // Ler todos os arquivos passados no formulário e acrescentar no Multipart como anexo
    for file in &data.files {
        let filename = file.file_name.clone().unwrap_or_else(|| "anexo".to_owned());
        let file_bytes = file.data.to_vec();

        // Conteúdo octet-stream são dados binários arbitrários
        let content_type = EmailContentType::parse("application/octet-stream").unwrap();
        let attachment = Attachment::new(filename.to_owned())
            .body(file_bytes.to_owned(), content_type);
        
        // Adicionar no construtor do Multipart
        multipart_body = multipart_body.singlepart(attachment);
    }

    // Ler dados do usuário da aplicação (.env) e de quem vai receber o email
    let user_email = dotenv!("EMAIL");
    let user_receiver = &data.to;

    let oauth_key = match cache_get_key::<&str, String>("GOOGLE_OAUTH2_KEY").await {
        Ok(access_token) => access_token,
        Err(_) => {
            let refresh_oauth2_token = refresh_oauth2_google().await;
            
            match refresh_oauth2_token {
                Ok(token) => token,
                Err(_) => {
                    // Preparar dados do erro interno para a resposta
                    let response = GenericError{
                        message: "Error refreshing OAuth2 token!",
                        error: "You have to regenerate OAuth2 code!"
                    };
                    
                    // Retornar dados com status 500
                    return HttpResponse::InternalServerError()
                        .content_type(ContentType::json())
                        .json(response);
                }
            }
        }
    };
    
    // Criar o Email
    let email = Message::builder()
        .from(user_email.parse().unwrap())
        .to(user_receiver.parse().unwrap())
        .subject(data.title.clone())
        .multipart(multipart_body).unwrap();

    // Resgatar as credenciais para conexão segura
    let creds = Credentials::new(user_email.to_owned(), 
    oauth_key);
    // Construtor do algoritmo de transporte pelo serviço do Gmail
    let mailer = SmtpTransport::starttls_relay("smtp.gmail.com").expect("Error creating StartTLS Transport")
        .authentication(vec![Mechanism::Xoauth2])
        .credentials(creds)
        .build();

    // Enviar email e verificar se o envio deu certo
    match mailer.send(&email) {
        Ok(_) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(EmailSentResponse {
                message: "Email sent successfully!",
                email: & EmailSendRequest {
                    title: data.title.to_string(),
                    content: data.content.to_string(),
                    to: data.to.to_string()
                }
            }),
        Err(e) => HttpResponse::BadGateway()
            .content_type(ContentType::json())
            .json(EmailSendError {
                message: "Error sending email!",
                error: &e.to_string()
            }),
    }

}

/// Endpoints de email
pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
web::scope("/email")
            .route("/send", web::post().to(send))
            .wrap(from_fn(auth_middleware))
    );
}