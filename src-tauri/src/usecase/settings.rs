use crate::domain::settings::Settings;
use crate::repository::diesel_settings_repository::SettingsRepository;

/// 設定を取得するユースケース
pub fn get_setting<R: SettingsRepository>(repo: &mut R) -> Result<Option<Settings>, String> {
    repo.get()
}

/// 設定を更新するユースケース
pub fn set_setting<R: SettingsRepository>(repo: &mut R, setting: Settings) -> Result<(), String> {
    repo.set(setting)
}
