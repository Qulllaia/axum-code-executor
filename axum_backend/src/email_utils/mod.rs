use std::env;
use dotenv::dotenv;

use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};

pub struct EmailUtils;
impl EmailUtils {
    pub async fn send_verification_email(email: &String, token: &String,) -> Result<(), Box<dyn std::error::Error>> { 
        dotenv().ok();
        
        let creds = Credentials::new(
            env::var("EMAIL_LOGIN").unwrap().to_string(),
            env::var("EMAIL_PASSWORD").unwrap().to_string()
        );

        let verification_link = format!("http://localhost:5000/verify?verify_token={}", token);
        let email = Message::builder()
        .from(env::var("EMAIL_LOGIN").unwrap().to_string().parse()?)
        .to(email.parse()?)
        .subject("Verify Your Account")
        .body(format!(
            "Click to verify: {}",
            verification_link
        ));

        let mailer = SmtpTransport::relay(env::var("SMTP_RELAY").unwrap().as_str())?
            .credentials(creds)
            .port(env::var("SMTP_PORT").unwrap().parse::<u16>().unwrap())
            .build();

        mailer.send(&email.unwrap())?;
        Ok(())
    }

}