use base64::{Engine as _, engine::general_purpose::STANDARD};
use chacha20poly1305::aead::rand_core::RngCore as _;
use once_cell::sync::OnceCell;
use thiserror::Error;

type PlatformError = Box<dyn std::error::Error + Send + Sync>;

static SERVICE: OnceCell<String> = OnceCell::new();
// Test override for key material. When set, `get_or_create_key` will return this.
#[cfg(test)]
static TEST_KEY_OVERRIDE: OnceCell<[u8; 32]> = OnceCell::new();

pub fn init_service(name: &str) {
    SERVICE.set(name.to_string()).ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_service_and_test_override() {
        init_service("test-service");
        let _ = KeyStore::set_test_key([7u8; 32]);
        let k = KeyStore::get_or_create_key().expect("got key");
        assert_eq!(k.len(), 32);
    }

    #[test]
    fn into_fixed_rejects_short_vector() {
        let v = vec![1u8; 10];
        let res = KeyStore::into_fixed(v);
        assert!(matches!(res, Err(KeystoreError::InvalidLength)));
    }

    #[test]
    fn generate_key_has_correct_length() {
        let k = KeyStore::generate_key();
        assert_eq!(k.len(), 32);
    }

    #[test]
    fn service_init_sets_value() {
        init_service("abc-service");
        // OnceCell may already have been initialized by other tests; just ensure it's non-empty.
        assert!(!service_str().is_empty());
    }
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
        // If test override has been set, use it.
        #[cfg(test)]
        {
            if let Some(k) = TEST_KEY_OVERRIDE.get() {
                return Ok(*k);
            }
        }
        #[cfg(target_os = "android")]
        {
            use keyring_core::Error as KeyringCoreError;
            // On Android, we have to use keyring_core crate instead of keyring crate.
            let entry = keyring_core::Entry::new(service_str(), Self::ACCOUNT)
                .map_err(KeystoreError::from)?;

            match entry.get_password() {
                Ok(b64) => {
                    let bytes = STANDARD
                        .decode(&b64)
                        .map_err(|e| KeystoreError::BadDataFormat(b64.into_bytes(), Box::new(e)))?;
                    return Self::into_fixed(bytes);
                }
                Err(KeyringCoreError::NoEntry) => {}
                Err(err) => return Err(KeystoreError::from(err)),
            }
            let key = Self::generate_key();
            let b64 = STANDARD.encode(key);
            entry.set_password(&b64).map_err(KeystoreError::from)?;
            return Ok(key);
        }

        #[cfg(not(target_os = "android"))]
        {
            let entry =
                keyring::Entry::new(service_str(), Self::ACCOUNT).map_err(KeystoreError::from)?;

            match entry.get_password() {
                Ok(b64) => {
                    let bytes = STANDARD
                        .decode(&b64)
                        .map_err(|e| KeystoreError::BadDataFormat(b64.into_bytes(), Box::new(e)))?;
                    return Self::into_fixed(bytes);
                }
                Err(keyring::Error::NoEntry) => {}
                Err(err) => return Err(KeystoreError::from(err)),
            }
            let key = Self::generate_key();
            let b64 = STANDARD.encode(key);
            entry.set_password(&b64).map_err(KeystoreError::from)?;
            Ok(key)
        }
    }

    /// Test helper: override the key returned by `get_or_create_key`.
    /// Only available in test builds.
    #[cfg(test)]
    pub fn set_test_key(key: [u8; 32]) {
        let _ = TEST_KEY_OVERRIDE.set(key);
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
