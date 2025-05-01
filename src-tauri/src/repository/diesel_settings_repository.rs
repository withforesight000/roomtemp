use crate::domain::settings::Settings;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sqlite::SqliteConnection;

// Diesel 用のスキーマ定義
pub mod schema {
    use diesel::table;

    table! {
        settings (id) {
            id -> Integer,
            url -> Text,
            access_token -> Text,
        }
    }
}

#[derive(Queryable)]
struct SettingEntity {
    pub id: i32,
    pub url: String,
    pub access_token: String,
}

#[derive(Insertable)]
#[diesel(table_name = schema::settings)]
struct NewSetting<'a> {
    pub id: i32,
    pub url: &'a str,
    pub access_token: &'a str,
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
        if let Some(entity) = result {
            Ok(Some(Settings {
                id: entity.id,
                url: entity.url,
                access_token: entity.access_token,
            }))
        } else {
            diesel::insert_into(settings)
                .values(&NewSetting {
                    id: 1,
                    url: "",
                    access_token: "",
                })
                .execute(&mut self.conn)
                .map_err(|e| e.to_string())?;

            Ok(Some(Settings {
                id: 1,
                url: "".to_string(),
                access_token: "".to_string(),
            }))
        }
    }

    fn set(&mut self, setting: Settings) -> Result<(), String> {
        use self::schema::settings::dsl::*;
        // 既存レコードの削除（簡易アップサート）
        diesel::delete(settings.filter(id.eq(1)))
            .execute(&mut self.conn)
            .map_err(|e| e.to_string())?;
        let new_setting = NewSetting {
            id: 1,
            url: &setting.url,
            access_token: &setting.access_token,
        };
        diesel::insert_into(settings)
            .values(&new_setting)
            .execute(&mut self.conn)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
