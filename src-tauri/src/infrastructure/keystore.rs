use base64::{engine::general_purpose::STANDARD, Engine as _};
use chacha20poly1305::aead::rand_core::RngCore as _;
use once_cell::sync::OnceCell;
use thiserror::Error;

type PlatformError = Box<dyn std::error::Error + Send + Sync>;

static SERVICE: OnceCell<String> = OnceCell::new();

pub fn init_service(name: &str) {
    SERVICE.set(name.to_string()).ok();
}

fn service_str() -> &'static str {
    SERVICE.get().expect("service name not initialized")
}

#[derive(Debug, Error)]
pub enum KeystoreError {
    #[error("platform secure storage failure: {0}")]
    PlatformFailure(#[source] PlatformError),
    #[error("couldn't access platform secure storage: {0}")]
    NoStorageAccess(#[source] PlatformError),
    #[error("no matching entry found in secure storage")]
    NoEntry,
    #[error("credential data is not UTF-8 encoded: {0:?}")]
    BadEncoding(Vec<u8>),
    #[error("credential data is not in the expected format: {1}")]
    BadDataFormat(Vec<u8>, #[source] PlatformError),
    #[error("attribute '{0}' is longer than the platform limit of {1} chars")]
    TooLong(String, u32),
    #[error("attribute {0} is invalid: {1}")]
    Invalid(String, String),
    #[error("entry is matched by {0} credentials in secure storage")]
    Ambiguous(usize),
    #[error("stored key has invalid length (expected 32 bytes)")]
    InvalidLength,
    #[error("unknown keystore error")]
    Other,
}

pub struct KeyStore;

#[cfg(not(target_os = "android"))]
impl From<keyring::Error> for KeystoreError {
    fn from(err: keyring::Error) -> Self {
        match err {
            keyring::Error::PlatformFailure(inner) => KeystoreError::PlatformFailure(inner),
            keyring::Error::NoStorageAccess(inner) => KeystoreError::NoStorageAccess(inner),
            keyring::Error::NoEntry => KeystoreError::NoEntry,
            keyring::Error::BadEncoding(bytes) => KeystoreError::BadEncoding(bytes),
            keyring::Error::TooLong(attr, limit) => KeystoreError::TooLong(attr, limit),
            keyring::Error::Invalid(attr, reason) => KeystoreError::Invalid(attr, reason),
            keyring::Error::Ambiguous(entries) => KeystoreError::Ambiguous(entries.len()),
            _ => KeystoreError::Other,
        }
    }
}

#[cfg(target_os = "android")]
impl From<keyring_core::Error> for KeystoreError {
    fn from(err: keyring_core::Error) -> Self {
        match err {
            keyring_core::Error::PlatformFailure(inner) => KeystoreError::PlatformFailure(inner),
            keyring_core::Error::NoStorageAccess(inner) => KeystoreError::NoStorageAccess(inner),
            keyring_core::Error::NoEntry => KeystoreError::NoEntry,
            keyring_core::Error::BadEncoding(bytes) => KeystoreError::BadEncoding(bytes),
            keyring_core::Error::TooLong(attr, limit) => KeystoreError::TooLong(attr, limit),
            keyring_core::Error::Invalid(attr, reason) => KeystoreError::Invalid(attr, reason),
            keyring_core::Error::Ambiguous(entries) => KeystoreError::Ambiguous(entries.len()),
            _ => KeystoreError::Other,
        }
    }
}

impl KeyStore {
    const ACCOUNT: &'static str = "encryption_key_v1";

    pub fn get_or_create_key() -> Result<[u8; 32], KeystoreError> {
        #[cfg(target_os = "android")]
        {
            use keyring_core::Error as KeyringCoreError;
            // On Android, we have to use keyring_core crate instead of keyring crate.
            let entry = keyring_core::Entry::new(service_str(), Self::ACCOUNT)
                .map_err(KeystoreError::from)?;

            match entry.get_password() {
                Ok(b64) => {
                    let bytes = STANDARD.decode(&b64).map_err(|e| {
                        KeystoreError::BadDataFormat(b64.into_bytes(), Box::new(e))
                    })?;
                    return Self::into_fixed(bytes);
                }
                Err(KeyringCoreError::NoEntry) => {}
                Err(err) => return Err(KeystoreError::from(err)),
            }
            let key = Self::generate_key();
            let b64 = STANDARD.encode(key);
            entry
                .set_password(&b64)
                .map_err(KeystoreError::from)?;
            return Ok(key);
        }

        #[cfg(not(target_os = "android"))]
        {
            let entry = keyring::Entry::new(service_str(), Self::ACCOUNT)
                .map_err(KeystoreError::from)?;

            match entry.get_password() {
                Ok(b64) => {
                    let bytes = STANDARD.decode(&b64).map_err(|e| {
                        KeystoreError::BadDataFormat(b64.into_bytes(), Box::new(e))
                    })?;
                    return Self::into_fixed(bytes);
                }
                Err(keyring::Error::NoEntry) => {}
                Err(err) => return Err(KeystoreError::from(err)),
            }
            let key = Self::generate_key();
            let b64 = STANDARD.encode(key);
            entry
                .set_password(&b64)
                .map_err(KeystoreError::from)?;
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
