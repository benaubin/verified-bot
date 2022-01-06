use ring::hmac;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifiedClaims {
    pub encrypted_eid: Vec<u8>,
    pub major: Vec<String>,
    pub school: Vec<String>,
    pub affiliation: Vec<String>
}


pub fn encode_token(claims: VerifiedClaims, shared_key: &[u8]) -> String {
    let mut data = rmp_serde::to_vec(&claims).unwrap();

    let hmac_key = hmac::Key::new(ring::hmac::HMAC_SHA256, shared_key);
    let hmac_tag = hmac::sign(&hmac_key, &data[..]);

    data.extend_from_slice(hmac_tag.as_ref());

    base64::encode_config(data, base64::URL_SAFE_NO_PAD)
}

#[derive(Debug)]
pub enum DecodeError {
  InvalidBase64(base64::DecodeError),
  InvalidMsgPack(rmp_serde::decode::Error),
  BadHmac
}

impl From<base64::DecodeError> for DecodeError {
    fn from(err: base64::DecodeError) -> Self {
        Self::InvalidBase64(err)
    }
}
impl From<rmp_serde::decode::Error> for DecodeError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        Self::InvalidMsgPack(err)
    }
}

pub fn decode_token(token: &str, shared_key: &[u8]) -> Result<VerifiedClaims, DecodeError> {
    let hmac_key = hmac::Key::new(ring::hmac::HMAC_SHA256, shared_key);

    let data = base64::decode_config(token, base64::URL_SAFE_NO_PAD)?;

    let (claims_raw, hmac_tag) = data.split_at(data.len() - 32);
    hmac::verify(&hmac_key, claims_raw, hmac_tag).map_err(|_| DecodeError::BadHmac)?;

    let claims = rmp_serde::from_read(claims_raw)?;
    Ok(claims)
}
