use crate::domain::settings::Setting;
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
    pub fn get(&mut self, key: String) -> Result<Option<String>, String> {
        let setting = settings::get_setting(self.repo, &key)?;
        Ok(setting.map(|s| s.value))
    }

    /// 設定の更新
    pub fn set(&mut self, key: String, value: String) -> Result<(), String> {
        let setting = Setting { key, value };
        settings::set_setting(self.repo, setting)
    }
}
