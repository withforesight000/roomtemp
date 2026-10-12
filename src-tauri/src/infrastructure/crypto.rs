use chacha20poly1305::aead::{Aead, Generate, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
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
        let key = Key::try_from(key_bytes).map_err(|_| CryptoError::InvalidKey)?;
        Ok(Self { key })
    }
}

impl Crypto for CryptoBox {
    fn encrypt_string(&self, plaintext: &str) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let cipher = ChaCha20Poly1305::new(&self.key);
        let nonce = Nonce::generate();
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
        let nonce = Nonce::try_from(nonce_bytes)
            .map_err(|_| CryptoError::InvalidNonceLen(nonce_bytes.len()))?;
        let pt = cipher
            .decrypt(&nonce, ciphertext)
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
    #[test]
    fn ciphertext_is_compatible_in_both_directions() {
        use chacha20poly1305_legacy::{
            ChaCha20Poly1305 as LegacyCipher, Nonce as LegacyNonce,
            aead::{Aead as _, KeyInit as _},
        };
        let key = [42u8; 32];
        let old = LegacyCipher::new_from_slice(&key).unwrap();
        let current = CryptoBox::new(&key).unwrap();
        let nonce = [17u8; 12];
        let plaintext = "existing-token-日本語";
        let ciphertext = old
            .encrypt(LegacyNonce::from_slice(&nonce), plaintext.as_bytes())
            .unwrap();
        assert_eq!(
            current.decrypt_string(&ciphertext, &nonce).unwrap(),
            plaintext
        );
        let (ciphertext, nonce) = current.encrypt_string(plaintext).unwrap();
        assert_eq!(
            old.decrypt(LegacyNonce::from_slice(&nonce), ciphertext.as_slice())
                .unwrap(),
            plaintext.as_bytes()
        );
        let mut corrupt = ciphertext.clone();
        corrupt[0] ^= 1;
        assert!(current.decrypt_string(&corrupt, &nonce).is_err());
        assert!(
            CryptoBox::new(&[43u8; 32])
                .unwrap()
                .decrypt_string(&ciphertext, &nonce)
                .is_err()
        );
    }
}
