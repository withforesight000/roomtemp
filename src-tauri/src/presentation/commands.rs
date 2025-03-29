use tauri::State;
use rand::distr::{Alphanumeric, SampleString};

use crate::infrastructure::db::AppState;
use crate::repository::diesel_settings_repository::DieselSettingsRepository;
use crate::controller::settings_controller::SettingsController;

// Tauri の tauri::generate_handler! マクロ経由で実際には使われている
#[allow(dead_code)]
#[tauri::command]
pub fn get_setting_command(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    let conn = state.pool.get().map_err(|e| e.to_string())?;
    let mut repo = DieselSettingsRepository { conn };
    let mut controller = SettingsController::new(&mut repo);
    controller.get(key)
}

#[allow(dead_code)]
#[tauri::command]
pub fn set_setting_command(state: State<AppState>, key: String, value: String) -> Result<(), String> {
    let conn = state.pool.get().map_err(|e| e.to_string())?;
    let mut repo = DieselSettingsRepository { conn };
    let mut controller = SettingsController::new(&mut repo);
    controller.set(key, value)
}

#[allow(dead_code)]
#[tauri::command]
pub fn my_custom_command(num: usize) -> String {
    let mut rng = rand::rng();
    Alphanumeric.sample_string(&mut rng, num)
}
