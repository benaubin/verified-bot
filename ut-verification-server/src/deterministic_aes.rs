//! Deterministic aes-gcm aead encryption
//! 
//! Based on ActiveRecord's strategy of using an hmac digest of
//! the encrypted plaintext as the encryption nonce.
//! 
//! ```
//! let key: [u8; 32] = rand::random();
//! 
//! let msg = b"bha366";
//! let encrypted = encrypt(msg, &key);
//! let decrypted = decrypt(&*encrypted, &key);
//! 
//! assert_eq!(msg, &*decrypted);
//! ```
//!

use aes_gcm::{NewAead, aead::Aead, Aes256Gcm};
use ring::hmac;

pub fn encrypt(msg: &[u8], key: &[u8]) -> Vec<u8> {
    // Based on the activerecord deterministic encryption algorithm
    // https://github.com/rails/rails/blob/main/activerecord/lib/active_record/encryption/cipher/aes256_gcm.rb
    let aes_key = aes_gcm::Key::from_slice(&key[..]);
    let cipher = Aes256Gcm::new(aes_key);

    let hmac_key = hmac::Key::new(ring::hmac::HMAC_SHA256, key);
    let hmac_tag = hmac::sign(&hmac_key, msg);
    let nonce = aes_gcm::Nonce::from_slice(&hmac_tag.as_ref()[0..12]);

    let mut ciphertext = cipher.encrypt(nonce, msg).unwrap();
    ciphertext.extend_from_slice(nonce);
    ciphertext
}

pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
  let aes_key = aes_gcm::Key::from_slice(&key[..]);
  let cipher = Aes256Gcm::new(aes_key);
  let (ciphertext, nonce) = ciphertext.split_at(ciphertext.len() - 12);
  let nonce = aes_gcm::Nonce::from_slice(nonce);
  cipher.decrypt(nonce, ciphertext)
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn with_random_msg() {
    let key: [u8; 32] = rand::random();

    for _ in 0..10 {
      let msg: [u8; 7] = rand::random();
      let encrypted = encrypt(&msg, &key);

      let decrypted = decrypt(&*encrypted, &key);

      assert_eq!(msg, &*decrypted);
    }
  }
}