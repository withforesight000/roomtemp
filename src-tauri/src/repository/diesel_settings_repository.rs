use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sqlite::SqliteConnection;
use crate::domain::settings::Setting;

// Diesel 用のスキーマ定義
pub mod schema {
    use diesel::table;

    table! {
        settings (key) {
            key -> Text,
            value -> Text,
        }
    }
}

#[derive(Queryable)]
struct SettingEntity {
    pub key: String,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name = schema::settings)]
struct NewSetting<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

/// リポジトリインターフェース（設定の取得・保存）
pub trait SettingsRepository {
    fn get(&mut self, key: &str) -> Result<Option<Setting>, String>;
    fn set(&mut self, setting: Setting) -> Result<(), String>;
}

/// Diesel を利用したリポジトリ実装
pub struct DieselSettingsRepository {
    pub conn: PooledConnection<ConnectionManager<SqliteConnection>>,
}

impl SettingsRepository for DieselSettingsRepository {
    fn get(&mut self, key_param: &str) -> Result<Option<Setting>, String> {
        use self::schema::settings::dsl::*;
        let result = settings
            .filter(key.eq(key_param))
            .first::<SettingEntity>(&mut self.conn)
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(entity) = result {
            Ok(Some(Setting {
                key: entity.key,
                value: entity.value,
            }))
        } else {
            Ok(None)
        }
    }

    fn set(&mut self, setting: Setting) -> Result<(), String> {
        use self::schema::settings::dsl::*;
        // 既存レコードの削除（簡易アップサート）
        diesel::delete(settings.filter(key.eq(&setting.key)))
            .execute(&mut self.conn)
            .map_err(|e| e.to_string())?;
        let new_setting = NewSetting {
            key: &setting.key,
            value: &setting.value,
        };
        diesel::insert_into(settings)
            .values(&new_setting)
            .execute(&mut self.conn)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
