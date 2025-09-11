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

/// リポジトリインターフェース（設定の取得・保存）
pub trait SettingsRepository {
    fn get(&mut self) -> Result<Option<Settings>, String>;
    fn set(&mut self, setting: Settings) -> Result<(), String>;
}

/// Diesel を利用したリポジトリ実装
pub struct DieselSettingsRepository {
    pub conn: PooledConnection<ConnectionManager<SqliteConnection>>,
}

impl SettingsRepository for DieselSettingsRepository {
    fn get(&mut self) -> Result<Option<Settings>, String> {
        use self::schema::settings::dsl::*;

        let result = settings
            .filter(id.eq(1))
            .first::<SettingEntity>(&mut self.conn)
            .optional()
            .map_err(|e| e.to_string())?;

        let key_bytes = KeyStore::get_or_create_key().map_err(|e| e.to_string())?;
        let crypt = CryptoBox::new(&key_bytes).map_err(|e| e.to_string())?;

        if let Some(entity) = result {
            let access_token = crypt
                .decrypt_string(
                    &entity.encrypted_access_token,
                    &entity.encrypted_access_token_nonce,
                )
                .map_err(|e| e.to_string())?;

            Ok(Some(Settings {
                id: entity.id,
                url: entity.url,
                access_token,
                use_proxies: entity.use_proxies,
                proxy_url: entity.proxy_url,
            }))
        } else {
        let (ciphertext, nonce) = crypt
            .encrypt_string("")
            .map_err(|e| e.to_string())?;

            diesel::insert_into(settings)
                .values(&NewSetting {
                    id: 1,
                    url: "",
                    encrypted_access_token: &ciphertext,
                    encrypted_access_token_nonce: &nonce,
                    use_proxies: false,
                    proxy_url: "",
                })
                .execute(&mut self.conn)
                .map_err(|e| e.to_string())?;

            Ok(Some(Settings {
                id: 1,
                url: "".to_string(),
                access_token: "".to_string(),
                use_proxies: false,
                proxy_url: "".to_string(),
            }))
        }
    }

    fn set(&mut self, setting: Settings) -> Result<(), String> {
        use self::schema::settings::dsl::*;

        let key_bytes = KeyStore::get_or_create_key().map_err(|e| e.to_string())?;
        let crypt = CryptoBox::new(&key_bytes).map_err(|e| e.to_string())?;

        let (ciphertext, nonce) = crypt
            .encrypt_string(&setting.access_token)
            .map_err(|e| e.to_string())?;

        // 既存レコードの削除（簡易アップサート）
        diesel::delete(settings.filter(id.eq(1)))
            .execute(&mut self.conn)
            .map_err(|e| e.to_string())?;
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
            .execute(&mut self.conn)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
