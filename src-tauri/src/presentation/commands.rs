use rand::distr::{Alphanumeric, SampleString};
use tauri::State;

use crate::controller::settings_controller::SettingsController;
use crate::domain::settings::Settings;
use crate::infrastructure::db::AppState;
use crate::repository::diesel_settings_repository::DieselSettingsRepository;

// Tauri の tauri::generate_handler! マクロ経由で実際には使われている
#[allow(dead_code)]
#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<Settings, String> {
    let conn = state.pool.get().map_err(|e| e.to_string())?;
    let mut repo = DieselSettingsRepository { conn };
    let mut controller = SettingsController::new(&mut repo);
    match controller.get() {
        Ok(settings) => Ok(settings.unwrap()),
        Err(e) => Err(e),
    }
}

#[allow(dead_code)]
#[tauri::command]
pub fn set_settings(
    state: State<AppState>,
    url: String,
    access_token: String,
) -> Result<(), String> {
    let conn = state.pool.get().map_err(|e| e.to_string())?;
    let mut repo = DieselSettingsRepository { conn };
    let mut controller = SettingsController::new(&mut repo);
    controller.set(url, access_token)
}

#[allow(dead_code)]
#[tauri::command]
pub fn my_custom_command(num: usize) -> String {
    let mut rng = rand::rng();
    Alphanumeric.sample_string(&mut rng, num)
}
