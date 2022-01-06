use handlebars::Handlebars;
use lettre::smtp::authentication::Credentials;
use lettre::{smtp::error::Error as SmtpError, SmtpClient, Transport};
use lettre_email::EmailBuilder;
use serde_json::json;

use utv_server::directory::Person;
use utv_token;

fn send_mail(name: &str, email: &str, token: &str) -> Result<(), SmtpError> {
    let from_address = std::env::var("FROM_ADDRESS").expect("Missing FROM_ADDRESS");
    let smtp_domain = std::env::var("SMTP_DOMAIN").expect("Missing SMTP_DOMAIN");
    let smtp_username = std::env::var("SMTP_USERNAME").expect("Missing SMTP_USERNAME");
    let smtp_password = std::env::var("SMTP_PASSWORD").expect("Missing SMTP_PASSWORD");

    static TEMPLATE: &'static str = include_str!("../email.hbs");
    let reg = Handlebars::new();
    let body = reg
        .render_template(
            TEMPLATE,
            &json!({
                "name": name,
                "token": token
            }),
        )
        .unwrap();

    let email = EmailBuilder::new()
        .to((email, name))
        .from(from_address)
        .subject("[Discord VerifiedBot] Verify your UT EID")
        .text(body)
        .build()
        .unwrap()
        .into();

    let smtp_creds = Credentials::new(smtp_username, smtp_password);
    let smtp_client = SmtpClient::new_simple(&smtp_domain)
        .unwrap()
        .credentials(smtp_creds);

    smtp_client.transport().send(email)?;

    Ok(())
}

fn main() {
    let shared_key = std::env::var("SHARED_KEY").expect("Missing SHARED_KEY");
    let encryption_key = std::env::var("ENCRYPTION_KEY").expect("Missing ENCRYPTION_KEY");

    let shared_key =
        base64::decode_config(shared_key, base64::URL_SAFE_NO_PAD).expect("Invalid ENCRYPTION_KEY");
    let encryption_key = base64::decode_config(encryption_key, base64::URL_SAFE_NO_PAD)
        .expect("Invalid ENCRYPTION_KEY");

    // TODO: authorize incoming requests
    let mut eid = String::new();
    if let Err(_) = std::io::stdin().read_line(&mut eid) {
        println!("Status: 400 Bad Request");
        return;
    }

    let person = match Person::lookup(eid.trim(), &encryption_key) {
        Ok(person) => person,
        Err(_) => {
            println!("Status: 400 Bad Request");
            return;
        }
    };

    let email = format!("{}@eid.utexas.edu", eid.trim());

    let token = utv_token::encode_token(&person.claims, &shared_key);

    let result = send_mail(&person.name, &person.email, &token);

    match result {
        Ok(_) => {
            println!("Status: 204 No Response");
            eprintln!("{}", token);
        }
        Err(err) => {
            println!("Status: 500 Internal Server Error");
            eprintln!("{:#?}", err);
        }
    }
}
