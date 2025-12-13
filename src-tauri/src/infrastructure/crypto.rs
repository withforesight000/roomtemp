use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{AeadCore as _, ChaCha20Poly1305, Key, Nonce};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("invalid key length (expected 32 bytes)")]
    InvalidKey,
    #[error("invalid nonce length (expected 12 bytes, got {0})")]
    InvalidNonceLen(usize),
    #[error("encryption failed")]
    Encrypt,
    #[error("decryption failed")]
    Decrypt,
}

pub trait Crypto {
    fn encrypt_string(&self, plaintext: &str) -> Result<(Vec<u8>, Vec<u8>), CryptoError>;
    fn decrypt_string(&self, ciphertext: &[u8], nonce_bytes: &[u8]) -> Result<String, CryptoError>;
}

pub struct CryptoBox {
    key: Key,
}

impl CryptoBox {
    pub fn new(key_bytes: &[u8]) -> Result<Self, CryptoError> {
        if key_bytes.len() != 32 {
            return Err(CryptoError::InvalidKey);
        }
        let key = Key::from_slice(key_bytes);
        Ok(Self { key: *key })
    }
}

impl Crypto for CryptoBox {
    fn encrypt_string(&self, plaintext: &str) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let cipher = ChaCha20Poly1305::new(&self.key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ct = cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| CryptoError::Encrypt)?;
        Ok((ct, nonce.to_vec()))
    }

    fn decrypt_string(&self, ciphertext: &[u8], nonce_bytes: &[u8]) -> Result<String, CryptoError> {
        if nonce_bytes.len() != 12 {
            return Err(CryptoError::InvalidNonceLen(nonce_bytes.len()));
        }
        let cipher = ChaCha20Poly1305::new(&self.key);
        let nonce = Nonce::from_slice(nonce_bytes);
        let pt = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::Decrypt)?;
        String::from_utf8(pt).map_err(|_| CryptoError::Decrypt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let key = [0u8; 32];
        let c = CryptoBox::new(&key).expect("valid key");
        let (ct, nonce) = c.encrypt_string("hello").expect("encrypt");
        let pt = c.decrypt_string(&ct, &nonce).expect("decrypt");
        assert_eq!(pt, "hello");
    }

    #[test]
    fn new_fails_for_bad_key_len() {
        let short = [0u8; 16];
        let res = CryptoBox::new(&short);
        assert!(matches!(res, Err(CryptoError::InvalidKey)));
    }

    #[test]
    fn decrypt_fails_for_bad_nonce_len() {
        let key = [0u8; 32];
        let c = CryptoBox::new(&key).expect("valid key");
        let (ct, _nonce) = c.encrypt_string("x").expect("encrypt");
        let res = c.decrypt_string(&ct, &[0u8; 8]);
        assert!(matches!(res, Err(CryptoError::InvalidNonceLen(8))));
    }
}
