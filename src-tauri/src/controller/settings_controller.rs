use crate::domain::settings::Settings;
use crate::repository::diesel_settings_repository::DieselSettingsRepository;
use crate::usecase::settings::{self, SettingsError};

/// コントローラーは、リクエスト（パラメータ）の検証や変換を行い、ユースケースを呼び出す役割を持ちます。
pub struct SettingsController<'a> {
    pub repo: &'a mut DieselSettingsRepository,
}

impl<'a> SettingsController<'a> {
    pub fn new(repo: &'a mut DieselSettingsRepository) -> Self {
        Self { repo }
    }

    /// 設定の取得（キーに対応する値を返す）
    pub fn get(&mut self) -> Result<Option<Settings>, SettingsError> {
        let setting = settings::get_setting(self.repo)?;
        Ok(setting)
    }

    /// 設定の更新
    pub fn set(
        &mut self,
        url: String,
        access_token: String,
        use_proxies: bool,
        proxy_url: String,
    ) -> Result<(), SettingsError> {
        let setting = Settings {
            id: 1,
            url,
            access_token,
            use_proxies,
            proxy_url,
        };
        settings::set_setting(self.repo, setting)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::keystore::KeyStore;
    use diesel::connection::SimpleConnection;
    use diesel::r2d2::ConnectionManager;
    use diesel::r2d2::Pool;
    use diesel::sqlite::SqliteConnection;

    fn make_repo() -> DieselSettingsRepository {
        KeyStore::set_test_key([9u8; 32]);
        // use a shared in-memory database so connections see the same state
        let database_url = "file:memdb1?mode=memory&cache=shared".to_string();
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder().build(manager).expect("pool");
        let mut conn = pool.get().expect("conn");
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
        DieselSettingsRepository { conn }
    }

    #[test]
    fn controller_set_and_get() {
        let mut repo = make_repo();
        let mut ctrl = SettingsController::new(&mut repo);

        ctrl.set("https://x".into(), "tok".into(), true, "http://p".into())
            .expect("set ok");

        let got = ctrl.get().expect("get ok").expect("some");
        assert_eq!(got.url, "https://x");
        assert!(got.use_proxies);
    }
}
