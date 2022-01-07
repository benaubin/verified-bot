use ring::hmac;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifiedClaims {
    pub encrypted_eid: Vec<u8>,
    pub major: Vec<String>,
    pub school: Vec<String>,
    pub affiliation: Vec<String>,
}

pub fn encode_token(claims: &VerifiedClaims, shared_key: &[u8]) -> String {
    let mut data = rmp_serde::to_vec(claims).unwrap();

    let hmac_key = hmac::Key::new(ring::hmac::HMAC_SHA256, shared_key);
    let hmac_tag = hmac::sign(&hmac_key, &data[..]);

    data.extend_from_slice(hmac_tag.as_ref());

    base64::encode_config(data, base64::URL_SAFE_NO_PAD)
}

#[derive(Debug)]
pub struct InvalidToken;

pub fn decode_token(token: &str, shared_key: &[u8]) -> Result<VerifiedClaims, InvalidToken> {
    let hmac_key = hmac::Key::new(ring::hmac::HMAC_SHA256, shared_key);

    let data = base64::decode_config(token, base64::URL_SAFE_NO_PAD).map_err(|_| InvalidToken)?;
    if data.len() <= 32 { return Err(InvalidToken) };

    let (claims_raw, hmac_tag) = data.split_at(data.len() - 32);
    hmac::verify(&hmac_key, claims_raw, hmac_tag).map_err(|_| InvalidToken)?;

    rmp_serde::from_read(claims_raw).map_err(|_| InvalidToken)
}
