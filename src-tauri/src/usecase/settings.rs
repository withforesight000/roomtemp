use thiserror::Error;

use crate::domain::settings::Settings;
use crate::repository::diesel_settings_repository::{
    DieselSettingsRepositoryError, SettingsRepository,
};

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error(transparent)]
    DieselSettingsRepository(#[from] DieselSettingsRepositoryError),
}

/// 設定を取得するユースケース
pub fn get_setting<R: SettingsRepository>(repo: &mut R) -> Result<Option<Settings>, SettingsError> {
    repo.get().map_err(SettingsError::DieselSettingsRepository)
}

/// 設定を更新するユースケース
pub fn set_setting<R: SettingsRepository>(
    repo: &mut R,
    setting: Settings,
) -> Result<(), SettingsError> {
    repo.set(setting)
        .map_err(SettingsError::DieselSettingsRepository)
}
