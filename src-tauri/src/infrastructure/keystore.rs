use base64::{Engine as _, engine::general_purpose::STANDARD};
use chacha20poly1305::aead::{Generate, Key};
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use thiserror::Error;

type PlatformError = Box<dyn std::error::Error + Send + Sync>;
static SERVICE: OnceCell<String> = OnceCell::new();
static KEY_LOCK: Mutex<()> = Mutex::new(());
#[cfg(test)]
static TEST_KEY_OVERRIDE: OnceCell<[u8; 32]> = OnceCell::new();

#[cfg(test)]
thread_local! {
    static TEST_KEY_CALLS: std::cell::RefCell<Vec<bool>> = const { std::cell::RefCell::new(Vec::new()) };
    static TEST_MISSING_KEY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub fn init_service(name: &str) {
    SERVICE.set(name.to_string()).ok();
}

/// Select the same credential identity used before the provider split.
pub fn init_default_store() -> Result<(), KeystoreError> {
    #[cfg(target_os = "macos")]
    let store = apple_native_keyring_store::keychain::Store::new()?;
    #[cfg(target_os = "ios")]
    let store = apple_native_keyring_store::protected::Store::new()?;
    #[cfg(target_os = "windows")]
    let store = windows_native_keyring_store::Store::new()?;
    #[cfg(target_os = "linux")]
    let store = linux_keyutils_keyring_store::Store::new()?;
    #[cfg(target_os = "android")]
    let store = android_native_keyring_store::Store::new_with_configuration(
        &std::collections::HashMap::from([("name", "net.withforesight.roomtemp.credentials.v1")]),
    )?;
    keyring_core::set_default_store(store);
    Ok(())
}

fn service_str() -> &'static str {
    SERVICE.get().expect("service name not initialized")
}

#[derive(Debug, Error)]
pub enum KeystoreError {
    #[error("platform secure storage failure")]
    PlatformFailure(#[source] PlatformError),
    #[error("couldn't access platform secure storage")]
    NoStorageAccess(#[source] PlatformError),
    #[error("no matching entry found in secure storage")]
    NoEntry,
    #[error("credential data is not UTF-8 encoded")]
    BadEncoding(Vec<u8>),
    #[error("credential data is not in the expected format")]
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

impl From<keyring_core::Error> for KeystoreError {
    fn from(err: keyring_core::Error) -> Self {
        match err {
            keyring_core::Error::PlatformFailure(inner) => KeystoreError::PlatformFailure(inner),
            keyring_core::Error::NoStorageAccess(inner) => KeystoreError::NoStorageAccess(inner),
            keyring_core::Error::NoEntry => KeystoreError::NoEntry,
            keyring_core::Error::BadEncoding(_) => KeystoreError::BadEncoding(Vec::new()),
            keyring_core::Error::BadDataFormat(_, error) => {
                KeystoreError::BadDataFormat(Vec::new(), error)
            }
            keyring_core::Error::TooLong(attr, limit) => KeystoreError::TooLong(attr, limit),
            keyring_core::Error::Invalid(attr, reason) => KeystoreError::Invalid(attr, reason),
            keyring_core::Error::Ambiguous(entries) => KeystoreError::Ambiguous(entries.len()),
            _ => KeystoreError::Other,
        }
    }
}

trait CredentialAccess {
    fn read(&self) -> Result<String, KeystoreError>;
    fn write(&self, password: &str) -> Result<(), KeystoreError>;
    fn delete(&self) -> Result<(), KeystoreError>;
}

impl CredentialAccess for keyring_core::Entry {
    fn read(&self) -> Result<String, KeystoreError> {
        self.get_password().map_err(Into::into)
    }
    fn write(&self, password: &str) -> Result<(), KeystoreError> {
        self.set_password(password).map_err(Into::into)
    }
    fn delete(&self) -> Result<(), KeystoreError> {
        self.delete_credential().map_err(Into::into)
    }
}

fn decode_key(password: &str) -> Result<[u8; 32], KeystoreError> {
    let bytes = STANDARD
        .decode(password)
        .map_err(|e| KeystoreError::BadDataFormat(Vec::new(), Box::new(e)))?;
    KeyStore::into_fixed(bytes)
}

// Caller serializes this whole operation, including write/read verification.
// Only NoEntry permits migration or creation; storage failures never do.
fn resolve_key(
    current: &dyn CredentialAccess,
    legacy: Option<&dyn CredentialAccess>,
    allow_create: bool,
) -> Result<[u8; 32], KeystoreError> {
    match current.read() {
        Ok(password) => return decode_key(&password),
        Err(KeystoreError::NoEntry) => {}
        Err(error) => return Err(error),
    }
    let old_key = match legacy {
        Some(old) => match old.read() {
            Ok(password) => Some(decode_key(&password)?),
            Err(KeystoreError::NoEntry) => None,
            Err(error) => return Err(error),
        },
        None => None,
    };
    let key = match old_key {
        Some(key) => key,
        None if allow_create => KeyStore::generate_key(),
        None => return Err(KeystoreError::NoEntry),
    };
    current.write(&STANDARD.encode(key))?;
    let verified = current.read().and_then(|password| decode_key(&password));
    match verified {
        Ok(stored) if stored == key => {}
        result => {
            // This entry was absent under KEY_LOCK. Remove an unverified write
            // so the next access can retry migration from the retained legacy key.
            match current.delete() {
                Ok(()) | Err(KeystoreError::NoEntry) => {}
                Err(error) => return Err(error),
            }
            return Err(result.err().unwrap_or(KeystoreError::Other));
        }
    }
    // Keep the legacy credential so an existing installation can roll back.
    Ok(key)
}

impl KeyStore {
    const ACCOUNT: &'static str = "encryption_key_v1";

    /// Read/migrate only. Existing ciphertext must never get a replacement key.
    pub fn get_key() -> Result<[u8; 32], KeystoreError> {
        Self::key(false)
    }

    /// Only call for an empty settings database.
    pub fn get_or_create_key() -> Result<[u8; 32], KeystoreError> {
        Self::key(true)
    }

    fn key(allow_create: bool) -> Result<[u8; 32], KeystoreError> {
        #[cfg(test)]
        {
            TEST_KEY_CALLS.with(|calls| calls.borrow_mut().push(allow_create));
            if TEST_MISSING_KEY.with(|missing| missing.get()) {
                return Err(KeystoreError::NoEntry);
            }
            if let Some(key) = TEST_KEY_OVERRIDE.get() {
                return Ok(*key);
            }
        }
        let _guard = KEY_LOCK.lock().map_err(|_| KeystoreError::Other)?;
        #[cfg(target_os = "linux")]
        let entry = keyring_core::Entry::new_with_modifiers(
            service_str(),
            Self::ACCOUNT,
            &std::collections::HashMap::from([(
                "description",
                format!("keyring-rs:{}@{}", Self::ACCOUNT, service_str()).as_str(),
            )]),
        )?;
        #[cfg(not(target_os = "linux"))]
        let entry = keyring_core::Entry::new(service_str(), Self::ACCOUNT)?;

        #[cfg(target_os = "android")]
        {
            use keyring_core::api::CredentialStoreApi;
            // Build the legacy entry only if the named store has no credential.
            match entry.read() {
                Ok(password) => return decode_key(&password),
                Err(KeystoreError::NoEntry) => {}
                Err(error) => return Err(error),
            }
            let old = android_native_keyring_store::LegacyStore::from_ndk_context()
                .map_err(keyring_core::Error::from)?;
            let old_entry = old.build(service_str(), Self::ACCOUNT, None)?;
            resolve_key(&entry, Some(&old_entry), allow_create)
        }
        #[cfg(not(target_os = "android"))]
        resolve_key(&entry, None, allow_create)
    }

    #[cfg(test)]
    pub fn set_test_key(key: [u8; 32]) {
        let _ = TEST_KEY_OVERRIDE.set(key);
    }

    #[cfg(test)]
    pub fn test_key_calls() -> Vec<bool> {
        TEST_KEY_CALLS.with(|calls| std::mem::take(&mut *calls.borrow_mut()))
    }

    #[cfg(test)]
    pub fn with_missing_test_key<T>(action: impl FnOnce() -> T) -> T {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                TEST_MISSING_KEY.with(|missing| missing.set(false));
            }
        }
        TEST_MISSING_KEY.with(|missing| missing.set(true));
        let _reset = Reset;
        action()
    }

    fn generate_key() -> [u8; 32] {
        Key::<chacha20poly1305::ChaCha20Poly1305>::generate().into()
    }

    fn into_fixed(bytes: Vec<u8>) -> Result<[u8; 32], KeystoreError> {
        bytes.try_into().map_err(|_| KeystoreError::InvalidLength)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[derive(Default)]
    struct MemoryEntry {
        password: Mutex<Option<String>>,
        writes: AtomicUsize,
        reads: AtomicUsize,
        fail_read: bool,
        fail_write: bool,
        corrupt_write: bool,
        fail_delete: bool,
    }
    impl MemoryEntry {
        fn with(key: [u8; 32]) -> Self {
            Self {
                password: Mutex::new(Some(STANDARD.encode(key))),
                ..Self::default()
            }
        }
    }
    impl CredentialAccess for MemoryEntry {
        fn read(&self) -> Result<String, KeystoreError> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            if self.fail_read {
                return Err(KeystoreError::Other);
            }
            self.password
                .lock()
                .unwrap()
                .clone()
                .ok_or(KeystoreError::NoEntry)
        }
        fn write(&self, password: &str) -> Result<(), KeystoreError> {
            self.writes.fetch_add(1, Ordering::SeqCst);
            if self.fail_write {
                return Err(KeystoreError::Other);
            }
            *self.password.lock().unwrap() = Some(if self.corrupt_write {
                STANDARD.encode([99u8; 32])
            } else {
                password.to_owned()
            });
            Ok(())
        }
        fn delete(&self) -> Result<(), KeystoreError> {
            if self.fail_delete {
                return Err(KeystoreError::NoStorageAccess(Box::new(
                    std::io::Error::other("delete failed"),
                )));
            }
            *self.password.lock().unwrap() = None;
            Ok(())
        }
    }
    #[test]
    fn migrates_and_verifies_legacy_key_without_deleting_it() {
        let current = MemoryEntry::default();
        let legacy = MemoryEntry::with([42; 32]);
        assert_eq!(
            resolve_key(&current, Some(&legacy), false).unwrap(),
            [42; 32]
        );
        assert_eq!(decode_key(&legacy.read().unwrap()).unwrap(), [42; 32]);
        assert_eq!(current.reads.load(Ordering::SeqCst), 2);
        assert_eq!(
            resolve_key(&current, Some(&legacy), false).unwrap(),
            [42; 32]
        );
        assert_eq!(current.writes.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn existing_named_key_wins_without_reading_legacy() {
        let current = MemoryEntry::with([7; 32]);
        let legacy = MemoryEntry {
            fail_read: true,
            ..MemoryEntry::default()
        };
        assert_eq!(resolve_key(&current, Some(&legacy), true).unwrap(), [7; 32]);
        assert_eq!(legacy.reads.load(Ordering::SeqCst), 0);
        assert_eq!(current.writes.load(Ordering::SeqCst), 0);
    }
    #[test]
    fn missing_key_for_existing_ciphertext_never_creates_one() {
        let current = MemoryEntry::default();
        let legacy = MemoryEntry::default();
        assert!(matches!(
            resolve_key(&current, Some(&legacy), false),
            Err(KeystoreError::NoEntry)
        ));
        assert_eq!(current.writes.load(Ordering::SeqCst), 0);
    }
    #[test]
    fn storage_errors_do_not_fall_back_or_overwrite() {
        let current = MemoryEntry {
            fail_read: true,
            ..MemoryEntry::default()
        };
        let legacy = MemoryEntry::with([42; 32]);
        assert!(resolve_key(&current, Some(&legacy), true).is_err());
        assert_eq!(legacy.reads.load(Ordering::SeqCst), 0);
        assert_eq!(current.writes.load(Ordering::SeqCst), 0);
        let current = MemoryEntry::default();
        let legacy = MemoryEntry {
            fail_read: true,
            ..MemoryEntry::default()
        };
        assert!(resolve_key(&current, Some(&legacy), true).is_err());
        assert_eq!(current.writes.load(Ordering::SeqCst), 0);
    }
    #[test]
    fn malformed_keys_are_not_overwritten_or_migrated() {
        for bad in ["not-base64".to_owned(), STANDARD.encode([1; 10])] {
            let corrupt = MemoryEntry {
                password: Mutex::new(Some(bad)),
                ..MemoryEntry::default()
            };
            let current = MemoryEntry::default();
            assert!(resolve_key(&current, Some(&corrupt), true).is_err());
            assert_eq!(current.writes.load(Ordering::SeqCst), 0);
            assert!(resolve_key(&corrupt, Some(&MemoryEntry::with([42; 32])), true).is_err());
            assert_eq!(corrupt.writes.load(Ordering::SeqCst), 0);
        }
    }
    #[test]
    fn migration_write_failures_and_readback_mismatch_preserve_legacy() {
        let old = MemoryEntry::with([42; 32]);
        for mut current in [
            MemoryEntry {
                fail_write: true,
                ..MemoryEntry::default()
            },
            MemoryEntry {
                corrupt_write: true,
                ..MemoryEntry::default()
            },
        ] {
            assert!(resolve_key(&current, Some(&old), false).is_err());
            assert_eq!(decode_key(&old.read().unwrap()).unwrap(), [42; 32]);
            assert!(matches!(current.read(), Err(KeystoreError::NoEntry)));
            current.fail_write = false;
            current.corrupt_write = false;
            assert_eq!(resolve_key(&current, Some(&old), false).unwrap(), [42; 32]);
        }
    }
    #[test]
    fn failed_verification_reports_cleanup_failure() {
        let current = MemoryEntry {
            corrupt_write: true,
            fail_delete: true,
            ..MemoryEntry::default()
        };
        assert!(matches!(
            resolve_key(&current, Some(&MemoryEntry::with([42; 32])), false),
            Err(KeystoreError::NoStorageAccess(_))
        ));
    }

    #[test]
    fn serialized_concurrent_first_access_returns_one_key() {
        let current = Arc::new(MemoryEntry::default());
        let workers: Vec<_> = (0..8)
            .map(|_| {
                let current = Arc::clone(&current);
                std::thread::spawn(move || {
                    let _guard = KEY_LOCK.lock().unwrap();
                    resolve_key(current.as_ref(), None, true).unwrap()
                })
            })
            .collect();
        let keys: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
        assert!(keys.iter().all(|key| key == &keys[0]));
        assert_eq!(current.writes.load(Ordering::SeqCst), 1);
    }

    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "writes only a unique test credential to the login keychain"]
    fn macos_legacy_provider_preserves_credential_identity() {
        use keyring_core::api::CredentialStoreApi;
        let service = format!("net.withforesight.roomtemp.test.{}", std::process::id());
        let old = keyring_legacy::Entry::new(&service, "migration-fixture").unwrap();
        struct Cleanup(keyring_legacy::Entry);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = self.0.delete_credential();
            }
        }
        let fixture = Cleanup(old);
        fixture.0.set_password("isolated-old-test-key").unwrap();
        let store = apple_native_keyring_store::keychain::Store::new().unwrap();
        let new = store.build(&service, "migration-fixture", None).unwrap();
        assert_eq!(new.get_password().unwrap(), "isolated-old-test-key");
        new.set_password("isolated-new-test-key").unwrap();
        assert_eq!(fixture.0.get_password().unwrap(), "isolated-new-test-key");
    }
}
