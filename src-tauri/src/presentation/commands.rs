use tauri::State;

use crate::app_state::AppState;
use crate::controller::settings_controller::SettingsController;
use crate::domain::settings::Settings;
use crate::infrastructure::grpc_client;
use crate::pb;
use crate::pb::tempgrpcd::TempgrpcdRequest;
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
pub async fn connect_to_grpc_server(state: State<'_, AppState>) -> Result<String, String> {
    let settings = get_settings(state.clone())?;

    if settings.url.is_empty() || settings.access_token.is_empty() {
        return Err("URL or access token is empty".into());
    }

    if state.grpc_connection.lock().await.is_some() {
        return Ok("Connected to gRPC server".into());
    }

    let client = grpc_client::new(&settings.url, &settings.access_token)
        .await
        .map_err(|e| format!("Failed to create gRPC client: {}", e))?;

    let mut guard = state.grpc_connection.lock().await;
    *guard = Some(client);

    Ok("Connected to gRPC server".into())
}

#[allow(dead_code)]
#[tauri::command]
pub async fn get_graph_data(
    state: State<'_, AppState>,
    start_time: u64,
    end_time: u64,
) -> Result<pb::tempgrpcd::TempgrpcdResponse, String> {
    let mut client = {
        let guard = state.grpc_connection.lock().await;
        // クローン可能なので clone してガードをすぐ手放す
        guard
            .as_ref()
            .cloned()
            .ok_or_else(|| "gRPC client is not connected".to_string())?
    };
    let resp = client
        .get_ambient_conditions(tonic::Request::new(TempgrpcdRequest {
            version: 1,
            start_time,
            end_time,
            samples: Some(1000),
        }))
        .await
        .map_err(|e| e.to_string())?;
    Ok(resp.into_inner())
}
