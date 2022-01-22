use handlebars::Handlebars;
use lettre_email::EmailBuilder;
use serde_json::json;
use serde::Deserialize;
use lazy_static::lazy_static;
use aws_sdk_sqs::{self, model::DeleteMessageBatchRequestEntry};
use ldap3::{LdapConnAsync, Ldap};

use utv_token;

use crate::directory::Person;
use mail_sender::MailSender;

mod deterministic_aes;
mod directory;
mod mail_sender;

lazy_static! {
    static ref SHARED_KEY: Vec<u8> = {
        let shared_key = std::env::var("SHARED_KEY").expect("Missing SHARED_KEY");
        base64::decode_config(shared_key, base64::URL_SAFE_NO_PAD).expect("Invalid SHARED_KEY")
    };
    static ref ENCRYPTION_KEY: Vec<u8> = {
        let encryption_key = std::env::var("ENCRYPTION_KEY").expect("Missing ENCRYPTION_KEY");
        base64::decode_config(encryption_key, base64::URL_SAFE_NO_PAD).expect("Invalid ENCRYPTION_KEY")
    };
    static ref FROM_ADDRESS: String = {
        std::env::var("FROM_ADDRESS").expect("Missing FROM_ADDRESS")
    };
    static ref SQS_VERIFICATION_REQUEST_URL: String = {
        "https://sqs.us-east-1.amazonaws.com/402762806873/eid_verification_requests".to_owned()
    };
}

static TEMPLATE: &'static str = include_str!("./email.hbs");
const REQUESTS_PER_SECOND: i32 = 10;

#[derive(Deserialize)]
struct VerificationRequest<'a> {
    eid: &'a str
}

async fn request_verification<'a>(mail_sender: &MailSender, ldap: &mut Ldap, req: VerificationRequest<'a>) {
    let eid = req.eid;
    let res = Person::lookup(ldap, eid, &ENCRYPTION_KEY).await;
    match res {
        Ok(person) => {
            let email = format!("{}@eid.utexas.edu", eid);
            let token = utv_token::encode_token(&person.claims, &SHARED_KEY);

            let reg = Handlebars::new();
            let body = reg
                .render_template(
                    TEMPLATE,
                    &json!({
                        "name": person.name,
                        "token": token
                    }),
                )
                .unwrap();

            let email = EmailBuilder::new()
                .to((email, person.name))
                .from(FROM_ADDRESS.as_str())
                .subject("[Discord VerifiedBot] Verify your UT EID")
                .text(body)
                .build()
                .unwrap()
                .into();

            if mail_sender.send(email).await.is_err() {
                panic!();
            }
        },
        Err(err) => {
            eprintln!("had a lookup error for eid {}: {:#?}", eid, err);
        }
    }

}

#[tokio::main]
async fn main() {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sqs::Client::new(&config);

    let mail_sender = mail_sender::spawn();

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let out = client
            .receive_message()
            .queue_url(SQS_VERIFICATION_REQUEST_URL.as_str())
            .max_number_of_messages(REQUESTS_PER_SECOND)
            .send()
            .await
            .unwrap();

        let messages = match out.messages {
            Some(msgs) => msgs,
            None => continue
        };

        let mut entries = Vec::new();

        let (conn, mut ldap) = LdapConnAsync::new("ldap://directory.utexas.edu:389").await.expect("failed to connect to directory");
        tokio::spawn(conn.drive());

        for msg in messages {
            let body = msg.body.expect("invalid message received");
            let req: VerificationRequest = serde_json::from_str(&body).expect("invalid message received");
            request_verification(&mail_sender, &mut ldap, req).await;
            entries.push(
                DeleteMessageBatchRequestEntry::builder()
                .set_id(Some(entries.len().to_string()))
                .set_receipt_handle( msg.receipt_handle)
                .build());
        }

        client
            .delete_message_batch()
            .queue_url(SQS_VERIFICATION_REQUEST_URL.as_str())
            .set_entries(Some(entries))
            .send()
            .await.unwrap();

        ldap.unbind().await.unwrap();
    }
}
