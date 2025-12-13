use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sqlite::SqliteConnection;

use crate::domain::settings::Settings;
use crate::infrastructure::crypto::{Crypto, CryptoBox};
use crate::infrastructure::keystore::KeyStore;

// Diesel 用のスキーマ定義
pub mod schema {
    use diesel::table;

    table! {
        settings (id) {
            id -> Integer,
            url -> Text,
            encrypted_access_token -> Blob,
            encrypted_access_token_nonce -> Blob,
            use_proxies -> Bool,
            proxy_url -> Text
        }
    }
}

#[derive(Queryable)]
struct SettingEntity {
    pub id: i32,
    pub url: String,
    pub encrypted_access_token: Vec<u8>,
    pub encrypted_access_token_nonce: Vec<u8>,
    pub use_proxies: bool,
    pub proxy_url: String,
}

#[derive(Insertable)]
#[diesel(table_name = schema::settings)]
struct NewSetting<'a> {
    pub id: i32,
    pub url: &'a str,
    pub encrypted_access_token: &'a [u8],
    pub encrypted_access_token_nonce: &'a [u8],
    pub use_proxies: bool,
    pub proxy_url: &'a str,
}

#[derive(Debug, thiserror::Error)]
pub enum DieselSettingsRepositoryError {
    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),
    #[error("crypto error: {0}")]
    Crypto(#[from] crate::infrastructure::crypto::CryptoError),
    #[error("keystore error: {0}")]
    Keystore(#[from] crate::infrastructure::keystore::KeystoreError),
}

/// リポジトリインターフェース（設定の取得・保存）
pub trait SettingsRepository {
    fn get(&mut self) -> Result<Option<Settings>, DieselSettingsRepositoryError>;
    fn set(&mut self, setting: Settings) -> Result<(), DieselSettingsRepositoryError>;
}

/// Diesel を利用したリポジトリ実装
pub struct DieselSettingsRepository {
    pub conn: PooledConnection<ConnectionManager<SqliteConnection>>,
}

impl SettingsRepository for DieselSettingsRepository {
    fn get(&mut self) -> Result<Option<Settings>, DieselSettingsRepositoryError> {
        use self::schema::settings::dsl::*;

        let result = settings
            .filter(id.eq(1))
            .first::<SettingEntity>(&mut self.conn)
            .optional()?;

        let key_bytes = KeyStore::get_or_create_key()?;
        let crypt = CryptoBox::new(&key_bytes)?;

        if let Some(entity) = result {
            let access_token = crypt.decrypt_string(
                &entity.encrypted_access_token,
                &entity.encrypted_access_token_nonce,
            )?;

            Ok(Some(Settings {
                id: entity.id,
                url: entity.url,
                access_token,
                use_proxies: entity.use_proxies,
                proxy_url: entity.proxy_url,
            }))
        } else {
            let (ciphertext, nonce) = crypt.encrypt_string("")?;

            diesel::insert_into(settings)
                .values(&NewSetting {
                    id: 1,
                    url: "",
                    encrypted_access_token: &ciphertext,
                    encrypted_access_token_nonce: &nonce,
                    use_proxies: false,
                    proxy_url: "",
                })
                .execute(&mut self.conn)?;

            Ok(Some(Settings {
                id: 1,
                url: "".to_string(),
                access_token: "".to_string(),
                use_proxies: false,
                proxy_url: "".to_string(),
            }))
        }
    }

    fn set(&mut self, setting: Settings) -> Result<(), DieselSettingsRepositoryError> {
        use self::schema::settings::dsl::*;

        let key_bytes = KeyStore::get_or_create_key()?;
        let crypt = CryptoBox::new(&key_bytes)?;

        let (ciphertext, nonce) = crypt.encrypt_string(&setting.access_token)?;

        // 既存レコードの削除（簡易アップサート）
        diesel::delete(settings.filter(id.eq(1))).execute(&mut self.conn)?;
        let new_setting = NewSetting {
            id: 1,
            url: &setting.url,
            encrypted_access_token: &ciphertext,
            encrypted_access_token_nonce: &nonce,
            use_proxies: setting.use_proxies,
            proxy_url: &setting.proxy_url,
        };
        diesel::insert_into(settings)
            .values(&new_setting)
            .execute(&mut self.conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::settings::Settings;
    use crate::infrastructure::keystore::KeyStore;
    use diesel::connection::SimpleConnection;
    use diesel::r2d2::ConnectionManager;
    use diesel::r2d2::Pool;
    use diesel::sqlite::SqliteConnection;
    use tempfile::NamedTempFile;

    #[test]
    fn set_and_get_setting_roundtrip() {
        // Override keystore to avoid platform dependency
        let mut key = [0u8; 32];
        key[0] = 42;
        KeyStore::set_test_key(key);
        // create a temporary file and keep it alive for the duration of the test
        let f = NamedTempFile::new().expect("temp file");
        let path = f.path().to_str().expect("path");
        let database_url = path.to_string();
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder().build(manager).expect("pool");
        let mut conn = pool.get().unwrap();
        // create table matching schema
        conn.batch_execute(
            "CREATE TABLE settings (
                id INTEGER PRIMARY KEY,
                url TEXT NOT NULL,
                encrypted_access_token BLOB NOT NULL,
                encrypted_access_token_nonce BLOB NOT NULL,
                use_proxies BOOLEAN NOT NULL,
                proxy_url TEXT NOT NULL
            );",
        )
        .unwrap();

        let conn = pool.get().unwrap();
        let mut repo = DieselSettingsRepository { conn };

        let s = Settings {
            id: 1,
            url: "https://example.com".into(),
            access_token: "tok".into(),
            use_proxies: true,
            proxy_url: "http://p".into(),
        };
        repo.set(s.clone()).expect("set ok");

        // New connection / repo to ensure persistence
        let conn2 = pool.get().unwrap();
        let mut repo2 = DieselSettingsRepository { conn: conn2 };
        let got = repo2.get().expect("get ok").expect("some");

        assert_eq!(got.url, "https://example.com");
        assert_eq!(got.use_proxies, true);
    }

    #[test]
    fn get_inserts_default_if_missing() {
        KeyStore::set_test_key([1u8; 32]);
        let f = NamedTempFile::new().expect("temp file");
        let path = f.path().to_str().expect("path");
        let database_url = path.to_string();
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder().build(manager).expect("pool");
        let mut conn = pool.get().unwrap();
        conn.batch_execute(
            "CREATE TABLE settings (
                id INTEGER PRIMARY KEY,
                url TEXT NOT NULL,
                encrypted_access_token BLOB NOT NULL,
                encrypted_access_token_nonce BLOB NOT NULL,
                use_proxies BOOLEAN NOT NULL,
                proxy_url TEXT NOT NULL
            );",
        )
        .unwrap();

        let conn = pool.get().unwrap();
        let mut repo = DieselSettingsRepository { conn };

        let maybe = repo.get().expect("get ok");
        assert!(maybe.is_some());
        let s = maybe.unwrap();
        assert_eq!(s.id, 1);
    }
}
