use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::header::ContentType;
use std::collections::HashMap;
use tera::{Context, Tera};

pub async fn send_email(
    to: &str,
    subject: &str,
    template_name: &str,
    context: &HashMap<&str, &str>,
    tera: &Tera,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tera_context = Context::new();
    for (key, value) in context {
        tera_context.insert(*key, value);
    }

    let email_body = tera.render(template_name, &tera_context)?;

    let email = Message::builder()
        .from("your_email@example.com".parse()?)
        .to(to.parse()?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(email_body)?;

    // let creds = Credentials::new("your_smtp_username".into(), "your_smtp_password".into());
    // let mailer = SmtpTransport::relay("smtp.example.com")?
    //     .credentials(creds)
    //     .build();
    // Configure the local Python SMTP debugging server as the SMTP server
    let mailer = SmtpTransport::builder_dangerous("localhost")
        .port(1025)
        .build();

    mailer.send(&email)?;

    Ok(())
}

