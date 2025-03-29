use crate::domain::settings::Setting;
use crate::repository::diesel_settings_repository::SettingsRepository;

/// 設定を取得するユースケース
pub fn get_setting<R: SettingsRepository>(repo: &mut R, key: &str) -> Result<Option<Setting>, String> {
    repo.get(key)
}

/// 設定を更新するユースケース
pub fn set_setting<R: SettingsRepository>(repo: &mut R, setting: Setting) -> Result<(), String> {
    repo.set(setting)
}
