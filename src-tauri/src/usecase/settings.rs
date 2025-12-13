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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::settings::Settings;
    use crate::infrastructure::crypto::CryptoError;
    use crate::repository::diesel_settings_repository::{
        DieselSettingsRepositoryError, SettingsRepository,
    };
    use mockall::mock;

    mock! {
        pub SettingsRepo {}
        impl SettingsRepository for SettingsRepo {
            fn get(&mut self) -> Result<Option<Settings>, DieselSettingsRepositoryError>;
            fn set(&mut self, setting: Settings) -> Result<(), DieselSettingsRepositoryError>;
        }
    }

    fn sample_setting() -> Settings {
        Settings {
            id: 1,
            url: "https://example.com".into(),
            access_token: "token-123".into(),
            use_proxies: false,
            proxy_url: "".into(),
        }
    }

    #[test]
    fn get_setting_returns_data() {
        let mut repo = MockSettingsRepo::new();
        repo.expect_get()
            .times(1)
            .returning(|| Ok(Some(sample_setting())));

        let settings = get_setting(&mut repo).expect("should return Ok");
        let settings = settings.expect("should have a setting record");

        assert_eq!(settings.url, "https://example.com");
        assert_eq!(settings.access_token, "token-123");
        assert!(!settings.use_proxies);
        assert_eq!(settings.proxy_url, "");
    }

    #[test]
    fn set_setting_persists_value() {
        let mut repo = MockSettingsRepo::new();
        repo.expect_set()
            .withf(|setting| {
                setting.url == "https://example.com"
                    && setting.access_token == "token-123"
                    && !setting.use_proxies
            })
            .times(1)
            .returning(|_| Ok(()));

        set_setting(&mut repo, sample_setting()).expect("should propagate success");
    }

    #[test]
    fn get_setting_propagates_repository_error() {
        let mut repo = MockSettingsRepo::new();
        repo.expect_get()
            .returning(|| Err(DieselSettingsRepositoryError::Crypto(CryptoError::Decrypt)));

        let err = get_setting(&mut repo).expect_err("should surface errors");
        match err {
            SettingsError::DieselSettingsRepository(DieselSettingsRepositoryError::Crypto(
                CryptoError::Decrypt,
            )) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
