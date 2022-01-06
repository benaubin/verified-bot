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
//! let decrypted = decrypt(&*encrypted, &key).unwrap();
//!
//! assert_eq!(msg, &*decrypted);
//! ```
//!

use aes_gcm_siv::{aead::{Aead, NewAead}, Aes256GcmSiv, Nonce};

pub fn encrypt(msg: &[u8], key: &[u8]) -> Vec<u8> {
    let aes_key = aes_gcm_siv::Key::from_slice(&key[..]);
    let cipher = Aes256GcmSiv::new(aes_key);
    let nonce = Nonce::from_slice(&[0; 12]);
    cipher.encrypt(nonce, msg).unwrap()
}

pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, ()> {
    let aes_key = aes_gcm_siv::Key::from_slice(&key[..]);
    let cipher = Aes256GcmSiv::new(aes_key);
    let nonce = Nonce::from_slice(&[0; 12]);
    cipher.decrypt(nonce, ciphertext).map_err(|_| ())
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

            let decrypted = decrypt(&*encrypted, &key).unwrap();

            assert_eq!(msg, &*decrypted);
        }
    }
}
