use utv_server::deterministic_aes;
use utv_server::directory::{LookupError, Person};

use serde::Serialize;
use serde_json;
use utv_token::VerifiedClaims;

fn main() { cgi::handle(|request| {
    let encryption_key = std::option_env!("ENCRYPTION_KEY").expect("Missing ENCRYPTION_KEY");
    let encryption_key = base64::decode_config(encryption_key, base64::URL_SAFE_NO_PAD)
        .expect("Invalid ENCRYPTION_KEY");
    let shared_key = std::option_env!("SHARED_KEY").expect("Missing SHARED_KEY");
    let shared_key =
        base64::decode_config(shared_key, base64::URL_SAFE_NO_PAD).expect("Invalid SHARED_KEY");

    let token = String::from_utf8(request.into_body());
    if let Err(_) = token {
        return cgi::text_response(400, "Token must be valid UTF8");
    }
    let token = token.unwrap();

    let old_claims = match utv_token::decode_token(token.trim(), &shared_key) {
        Ok(c) => c,
        Err(_) => {
            return cgi::text_response(400, "Bad Token");
        }
    };

    let eid = match deterministic_aes::decrypt(&old_claims.encrypted_eid, &encryption_key) {
        Ok(eid) => eid,
        Err(_) => {
            return cgi::text_response(400, "Bad encrypted EID");
        }
    };
    let eid = std::str::from_utf8(&eid[..])
        .expect("encrypted and authenticated eid should be valid utf8");

    let person = match Person::lookup(&eid, &encryption_key) {
        Ok(person) => person,
        Err(LookupError::NotFound) => {
            return cgi::text_response(404, "User no longer exists");
        }
        Err(LookupError::MissingDirectoryInfo(_)) => {
            return cgi::text_response(422, "Required directory info not found");
        }
        Err(LookupError::LdapError(_)) => {
            return cgi::text_response(500, "LDAP failed");
        }
    };

    let new_token = utv_token::encode_token(&person.claims, &shared_key);

    cgi::binary_response(200, "application/json;charset=UTF-8", serde_json::to_vec(&Response {
        token: new_token,
        claims: person.claims
    }).unwrap())
})}

#[derive(Serialize)]
struct Response {
    token: String,
    claims: VerifiedClaims,
}
