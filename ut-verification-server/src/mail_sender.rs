use lettre::{SendableEmail};
use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, Transport};

pub type MailSender = tokio::sync::mpsc::Sender<SendableEmail>;

pub fn spawn() -> MailSender {
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);

    std::thread::spawn(move || {
        let mut transport = {
            let smtp_user = std::env::var("SMTP_USERNAME").expect("Missing SHARED_KEY");
            let smtp_pass = std::env::var("SMTP_PASSWORD").expect("Missing SMTP_PASSWORD");
            let smtp_domain = std::env::var("SMTP_DOMAIN").expect("Missing SMTP_DOMAIN");
            let smtp_creds = Credentials::new(smtp_user, smtp_pass);
            SmtpClient::new_simple(&smtp_domain).unwrap().credentials(smtp_creds).transport()
        };
        while let Some(mail) = rx.blocking_recv() {
            println!("Sent mail, status: {:?}", transport.send(mail));
        }
    });

    tx
}
