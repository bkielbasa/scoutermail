use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SmtpError {
    #[error("failed to send email: {0}")]
    Send(String),
    #[error("invalid email address: {0}")]
    Address(String),
}

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct ComposeEmail {
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
}

pub async fn send_email(config: &SmtpConfig, email: &ComposeEmail) -> Result<(), SmtpError> {
    let from = email
        .from
        .parse::<lettre::message::Mailbox>()
        .map_err(|e| SmtpError::Address(format!("invalid from address '{}': {}", email.from, e)))?;

    let mut builder = Message::builder().from(from).subject(&email.subject);

    for addr in &email.to {
        let mailbox = addr
            .parse::<lettre::message::Mailbox>()
            .map_err(|e| SmtpError::Address(format!("invalid to address '{}': {}", addr, e)))?;
        builder = builder.to(mailbox);
    }

    for addr in &email.cc {
        let mailbox = addr
            .parse::<lettre::message::Mailbox>()
            .map_err(|e| SmtpError::Address(format!("invalid cc address '{}': {}", addr, e)))?;
        builder = builder.cc(mailbox);
    }

    for addr in &email.bcc {
        let mailbox = addr
            .parse::<lettre::message::Mailbox>()
            .map_err(|e| SmtpError::Address(format!("invalid bcc address '{}': {}", addr, e)))?;
        builder = builder.bcc(mailbox);
    }

    if let Some(ref reply_to) = email.in_reply_to {
        builder = builder.in_reply_to(reply_to.clone());
    }

    if !email.references.is_empty() {
        builder = builder.references(email.references.join(" "));
    }

    let message = if let Some(ref html) = email.body_html {
        builder
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(email.body_text.clone()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(html.clone()),
                    ),
            )
            .map_err(|e| SmtpError::Send(format!("failed to build multipart message: {}", e)))?
    } else {
        builder
            .body(email.body_text.clone())
            .map_err(|e| SmtpError::Send(format!("failed to build message: {}", e)))?
    };

    let creds = Credentials::new(config.username.clone(), config.password.clone());

    // Port 465 uses implicit TLS; port 587/25 use STARTTLS
    let transport = if config.port == 465 {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
            .map_err(|e| SmtpError::Send(format!("failed to create SMTP transport: {}", e)))?
            .port(config.port)
            .credentials(creds)
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
            .map_err(|e| SmtpError::Send(format!("failed to create SMTP transport: {}", e)))?
            .port(config.port)
            .credentials(creds)
            .build()
    };

    transport
        .send(message)
        .await
        .map_err(|e| SmtpError::Send(format!("failed to send email: {}", e)))?;

    Ok(())
}
