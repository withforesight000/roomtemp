use base64::{Engine as _, engine::general_purpose::STANDARD};
use chacha20poly1305::aead::rand_core::RngCore as _;
use once_cell::sync::OnceCell;
use thiserror::Error;

static SERVICE: OnceCell<String> = OnceCell::new();

pub fn init_service(name: &str) {
    SERVICE.set(name.to_string()).ok();
}

fn service_str() -> &'static str {
    SERVICE.get().expect("service name not initialized")
}

#[derive(Debug, Error)]
pub enum KeystoreError {
    #[error("secure storage error: {0}")]
    Other(String),
    #[error("stored key has invalid length (expected 32 bytes)")]
    InvalidLength,
}

pub struct KeyStore;

impl KeyStore {
    const ACCOUNT: &'static str = "encryption_key_v1";

    pub fn get_or_create_key() -> Result<[u8; 32], KeystoreError> {
        #[cfg(target_os = "android")]
        {
            // On Android, We have to use keyring_core crate instead of keyring crate.
            use keyring_core;
            let entry = keyring_core::Entry::new(service_str(), Self::ACCOUNT)
                .map_err(|e| KeystoreError::Other(e.to_string()))?;

            if let Ok(b64) = entry.get_password() {
                let bytes = STANDARD
                    .decode(&b64)
                    .map_err(|e| KeystoreError::Other(e.to_string()))?;
                return Self::into_fixed(bytes);
            }
            let key = Self::generate_key();
            let b64 = STANDARD.encode(key);
            entry
                .set_password(&b64)
                .map_err(|e| KeystoreError::Other(e.to_string()))?;
            return Ok(key);
        }

        #[cfg(not(target_os = "android"))]
        {
            let entry = keyring::Entry::new(service_str(), Self::ACCOUNT)
                .map_err(|e| KeystoreError::Other(e.to_string()))?;

            if let Ok(b64) = entry.get_password() {
                let bytes = STANDARD
                    .decode(&b64)
                    .map_err(|e| KeystoreError::Other(e.to_string()))?;
                return Self::into_fixed(bytes);
            }
            let key = Self::generate_key();
            let b64 = STANDARD.encode(key);
            entry
                .set_password(&b64)
                .map_err(|e| KeystoreError::Other(e.to_string()))?;
            Ok(key)
        }
    }

    fn generate_key() -> [u8; 32] {
        use chacha20poly1305::aead::rand_core::OsRng;
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    fn into_fixed(bytes: Vec<u8>) -> Result<[u8; 32], KeystoreError> {
        if bytes.len() != 32 {
            return Err(KeystoreError::InvalidLength);
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(key)
    }
}
