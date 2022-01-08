use std::collections::HashSet;

use handlebars::Handlebars;
use jsonwebtoken::Validation;
use lettre::smtp::authentication::Credentials;
use lettre::{smtp::error::Error as SmtpError, SmtpClient, Transport};
use lettre_email::EmailBuilder;
use serde::Deserialize;
use serde_json::json;

use utv_server::directory::Person;
use utv_token;

fn send_mail(name: &str, email: &str, token: &str) -> Result<(), SmtpError> {
    let from_address = std::option_env!("FROM_ADDRESS").expect("Missing FROM_ADDRESS");
    let smtp_domain = std::option_env!("SMTP_DOMAIN").expect("Missing SMTP_DOMAIN");
    let smtp_username = std::option_env!("SMTP_USERNAME").expect("Missing SMTP_USERNAME");
    let smtp_password = std::option_env!("SMTP_PASSWORD").expect("Missing SMTP_PASSWORD");

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

    let smtp_creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());
    let smtp_client = SmtpClient::new_simple(&smtp_domain)
        .unwrap()
        .credentials(smtp_creds);

    smtp_client.transport().send(email)?;

    Ok(())
}


#[derive(Deserialize)]
struct RequestClaims {
    ut_eid: String
}

fn main() {
    let shared_key = std::option_env!("SHARED_KEY").expect("Missing SHARED_KEY");
    let encryption_key = std::option_env!("ENCRYPTION_KEY").expect("Missing ENCRYPTION_KEY");
    let request_key = std::option_env!("REQUEST_KEY").expect("Missing REQUEST_KEY");
    let shared_key =
        base64::decode_config(shared_key, base64::URL_SAFE_NO_PAD).expect("Invalid ENCRYPTION_KEY");
    let encryption_key = base64::decode_config(encryption_key, base64::URL_SAFE_NO_PAD)
        .expect("Invalid ENCRYPTION_KEY");
    let request_key = base64::decode_config(&request_key, base64::URL_SAFE_NO_PAD).expect("Invalid REQUEST_KEY");
    let request_key = jsonwebtoken::DecodingKey::from_secret(&request_key);

    let mut line = String::new();
    loop {
        if let Err(err) = std::io::stdin().read_line(&mut line) {
            panic!("{:?}", err);
        }
        if line.trim().len() == 0 { break; }
        line.clear();
    }

    let mut request_token = String::new();
    if let Err(_) = std::io::stdin().read_line(&mut request_token) {
        println!("Status: 400 Bad Request\n");
        return;
    }

    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.aud = Some(HashSet::from_iter(["ut-verification-server".to_owned()]));
    validation.sub = Some("request-token".to_owned());
    let request: RequestClaims = match jsonwebtoken::decode(request_token.trim(), &request_key, &validation) {
        Ok(token) => token.claims,
        Err(_) => {
            println!("Status: 400 Bad Request\n");
            return;
        }
    };

    let eid = request.ut_eid;
    let person = match Person::lookup(&eid, &encryption_key) {
        Ok(person) => person,
        Err(_) => {
            println!("Status: 400 Bad Request\n");
            return;
        }
    };

    let email = format!("{}@eid.utexas.edu", eid);
    let token = utv_token::encode_token(&person.claims, &shared_key);

    match send_mail(&person.name, &email, &token) {
        Ok(_) => {
            println!("Status: 200 OK\n");
        }
        Err(_) => {
            println!("Status: 500 Internal Server Error\n");
        }
    }
}
