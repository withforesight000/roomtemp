use crate::domain::settings::Settings;
use crate::repository::diesel_settings_repository::DieselSettingsRepository;
use crate::usecase::settings;

/// コントローラーは、リクエスト（パラメータ）の検証や変換を行い、ユースケースを呼び出す役割を持ちます。
pub struct SettingsController<'a> {
    pub repo: &'a mut DieselSettingsRepository,
}

impl<'a> SettingsController<'a> {
    pub fn new(repo: &'a mut DieselSettingsRepository) -> Self {
        Self { repo }
    }

    /// 設定の取得（キーに対応する値を返す）
    pub fn get(&mut self) -> Result<Option<Settings>, String> {
        let setting = settings::get_setting(self.repo)?;
        Ok(setting)
    }

    /// 設定の更新
    pub fn set(&mut self, url: String, access_token: String, use_proxies: bool, proxy_url: String) -> Result<(), String> {
        let setting = Settings { id: 1, url, access_token, use_proxies, proxy_url };
        settings::set_setting(self.repo, setting)
    }
}
