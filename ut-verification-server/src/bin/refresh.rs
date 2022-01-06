use utv_server::deterministic_aes;
use utv_server::directory::{LookupError, Person};

use serde::Serialize;
use serde_json;
use utv_token::VerifiedClaims;

fn main() {
    let encryption_key = std::env::var("SHARED_KEY").expect("Missing ENCRYPTION_KEY");
    let encryption_key = base64::decode_config(encryption_key, base64::URL_SAFE_NO_PAD)
        .expect("Invalid ENCRYPTION_KEY");
    let shared_key = std::env::var("SHARED_KEY").expect("Missing SHARED_KEY");
    let shared_key =
        base64::decode_config(shared_key, base64::URL_SAFE_NO_PAD).expect("Invalid SHARED_KEY");

    let mut token = String::new();
    if let Err(_) = std::io::stdin().read_line(&mut token) {
        println!("Status: 400 Bad Request");
        return;
    }

    let old_claims = match utv_token::decode_token(token.trim(), &shared_key) {
        Ok(c) => c,
        Err(_) => {
            println!("Status: 400 Bad Request");
            println!();
            println!("Bad token");
            return;
        }
    };

    let eid = match deterministic_aes::decrypt(&old_claims.encrypted_eid, &encryption_key) {
        Ok(eid) => eid,
        Err(_) => {
            println!("Status: 400 Bad Request");
            println!();
            println!("Bad encrypted eid");
            return;
        }
    };
    let eid = std::str::from_utf8(&eid[..])
        .expect("encrypted and authenticated eid should be valid utf8");

    let person = match Person::lookup(&eid, &encryption_key) {
        Ok(person) => person,
        Err(LookupError::NotFound) => {
            println!("Status: 404 Not Found");
            println!("Content-Type: text/plain;charset=UTF-8");
            println!();
            println!("User no longer exists");
            return;
        }
        Err(LookupError::MissingDirectoryInfo(_)) => {
            println!("Status: 422 Unprocessable Entity");
            println!("Content-Type: text/plain;charset=UTF-8");
            println!();
            println!("Required directory info not found");
            return;
        }
        Err(LookupError::LdapError(_)) => {
            println!("Status: 500 Internal Server Error");
            println!("Content-Type: text/plain;charset=UTF-8");
            println!();
            println!("Ldap Failed");
            return;
        }
    };

    let new_token = utv_token::encode_token(&person.claims, &shared_key);

    println!("Status: 200 OK");
    println!("Content-Type: application/json;charset=UTF-8");
    println!();
    serde_json::to_writer(
        std::io::stdout(),
        &Response {
            token: new_token,
            claims: person.claims,
        },
    )
    .unwrap();
}

#[derive(Serialize)]
struct Response {
    token: String,
    claims: VerifiedClaims,
}
