use base64::{Engine as _, engine::general_purpose::STANDARD};
use crypto::{Crypto, CryptoBox};
use diesel::{Connection, RunQueryDsl, SqliteConnection};
use jni::{
    JNIEnv,
    objects::{JClass, JObject, JString},
    sys::jstring,
};
use std::path::{Path, PathBuf};

#[derive(diesel::QueryableByName)]
struct Ciphertext {
    #[diesel(sql_type = diesel::sql_types::Binary)]
    encrypted_access_token: Vec<u8>,
    #[diesel(sql_type = diesel::sql_types::Binary)]
    encrypted_access_token_nonce: Vec<u8>,
}
fn database(root: &Path, depth: usize) -> Option<PathBuf> {
    if depth == 0 {
        return None;
    }
    for item in std::fs::read_dir(root).ok()?.flatten() {
        let path = item.path();
        if path.file_name()? == "roomtemp.db" {
            return Some(path);
        }
        if path.is_dir() && !matches!(path.file_name()?.to_str()?, "cache" | "code_cache" | "lib") {
            if let Some(path) = database(&path, depth - 1) {
                return Some(path);
            }
        }
    }
    None
}
fn probe(data_dir: &Path, mode: &str) -> Result<String, &'static str> {
    use keyring_core_old::api::CredentialStoreApi as _;
    let service = "net.withforesight.roomtemp";
    let old_store =
        android_store_old::AndroidStore::from_ndk_context().map_err(|_| "old_store_init")?;
    let old = old_store
        .build(service, "encryption_key_v1", None)
        .map_err(|_| "old_entry_build")?;
    let old_password = old.get_password().map_err(|_| "old_key_read")?;
    let old_key: [u8; 32] = STANDARD
        .decode(&old_password)
        .map_err(|_| "old_base64")?
        .try_into()
        .map_err(|_| "old_key_length")?;
    let db = database(data_dir, 6).ok_or("existing_database_not_found")?;
    let db_before = std::fs::read(&db).map_err(|_| "database_snapshot")?;
    let legacy_file = data_dir.join("shared_prefs/net.withforesight.roomtemp.xml");
    let legacy_before = std::fs::read(&legacy_file).map_err(|_| "legacy_preferences_snapshot")?;
    let mut conn = SqliteConnection::establish(&format!("file:{}?mode=ro", db.display()))
        .map_err(|_| "database_readonly_open")?;
    let rows: Vec<Ciphertext> = diesel::sql_query(
        "SELECT encrypted_access_token, encrypted_access_token_nonce FROM settings WHERE id = 1",
    )
    .load(&mut conn)
    .map_err(|_| "ciphertext_read")?;
    if rows.len() != 1 {
        return Err("expected_existing_settings_row");
    }
    let row = &rows[0];
    let old_plaintext = CryptoBox::new(&old_key)
        .map_err(|_| "old_crypto_key")?
        .decrypt_string(
            &row.encrypted_access_token,
            &row.encrypted_access_token_nonce,
        )
        .map_err(|_| "existing_ciphertext_decrypt")?;
    if old_plaintext.is_empty() {
        return Err("existing_access_token_empty");
    }
    keystore::init_service(service);
    keystore::init_default_store().map_err(|_| "named_store_init")?;
    let named =
        keyring_core::Entry::new(service, "encryption_key_v1").map_err(|_| "named_entry_build")?;
    let named_present = match named.get_password() {
        Ok(_) => true,
        Err(keyring_core::Error::NoEntry) => false,
        Err(_) => return Err("named_key_read_before"),
    };
    if mode == "migrate" && named_present {
        return Err("migration_precondition_named_key_already_present");
    }
    if mode == "verify" && !named_present {
        return Err("persistence_precondition_named_key_missing");
    }
    let new_key = keystore::KeyStore::get_key().map_err(|_| "production_get_key")?;
    if new_key != old_key {
        return Err("migrated_key_mismatch");
    }
    let named_password = named.get_password().map_err(|_| "named_key_read_after")?;
    if named_password != old_password {
        return Err("named_password_mismatch");
    }
    let new_plaintext = CryptoBox::new(&new_key)
        .map_err(|_| "new_crypto_key")?
        .decrypt_string(
            &row.encrypted_access_token,
            &row.encrypted_access_token_nonce,
        )
        .map_err(|_| "migrated_ciphertext_decrypt")?;
    if new_plaintext != old_plaintext {
        return Err("plaintext_mismatch");
    }
    if old.get_password().map_err(|_| "legacy_key_retained_read")? != old_password {
        return Err("legacy_key_changed");
    }
    if std::fs::read(&db).map_err(|_| "database_after")? != db_before {
        return Err("database_changed");
    }
    if std::fs::read(&legacy_file).map_err(|_| "legacy_after")? != legacy_before {
        return Err("legacy_preferences_changed");
    }
    Ok(format!(
        "PASS mode={mode};old_provider=0.4.0;named_before={named_present};key_equal=true;existing_token_nonempty=true;plaintext_equal=true;legacy_retained=true;database_unchanged=true;legacy_preferences_unchanged=true"
    ))
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_net_withforesight_roomtemp_migrationprobe_MigrationProbe_runProbe(
    mut env: JNIEnv,
    _class: JClass,
    context: JObject,
    data_dir: JString,
    mode: JString,
) -> jstring {
    let outcome = (|| -> Result<String, &'static str> {
        let data: String = env
            .get_string(&data_dir)
            .map_err(|_| "jni_data_dir")?
            .into();
        let mode: String = env.get_string(&mode).map_err(|_| "jni_mode")?.into();
        if mode != "migrate" && mode != "verify" {
            return Err("invalid_mode");
        }
        let vm = env.get_java_vm().map_err(|_| "jni_vm")?;
        let context = env.new_global_ref(context).map_err(|_| "jni_context")?;
        unsafe {
            ndk_context::initialize_android_context(
                vm.get_java_vm_pointer().cast(),
                context.as_obj().as_raw().cast(),
            );
        }
        let result = probe(Path::new(&data), &mode);
        keyring_core::unset_default_store();
        unsafe {
            ndk_context::release_android_context();
        }
        result
    })();
    let message = outcome.unwrap_or_else(|stage| format!("FAIL stage={stage}"));
    env.new_string(message)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}
