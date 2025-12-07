use pbjson_types::Timestamp;
use prost::Message;
use tauri::State;
use tauri::ipc::Response;
use tempgrpcd_protos::tempgrpcd::v1::GetAmbientConditionsRequest;

use crate::app_state::AppState;
use crate::controller::settings_controller::SettingsController;
use crate::domain::settings::Settings;
use crate::infrastructure::grpc_client;
use crate::presentation::ui_error::{self, UIError};
use crate::repository::diesel_settings_repository::DieselSettingsRepository;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<Settings, UIError> {
    let conn = state.pool.get()?;
    let mut repo = DieselSettingsRepository { conn };
    let mut controller = SettingsController::new(&mut repo);
    match controller.get() {
        Ok(settings) => Ok(settings.unwrap()),
        Err(e) => Err(UIError::from(e)),
    }
}

#[tauri::command]
pub fn set_settings(
    state: State<AppState>,
    url: String,
    access_token: String,
    use_proxies: bool,
    proxy_url: String,
) -> Result<(), UIError> {
    let conn = state.pool.get()?;
    let mut repo = DieselSettingsRepository { conn };
    let mut controller = SettingsController::new(&mut repo);
    controller.set(url, access_token, use_proxies, proxy_url)?;
    Ok(())
}

#[tauri::command]
pub async fn connect_to_grpc_server(state: State<'_, AppState>) -> Result<String, UIError> {
    let settings = get_settings(state.clone())?;

    if settings.url.is_empty() || settings.access_token.is_empty() {
        return Err(ui_error::url_access_token_empty_error());
    }

    let client = grpc_client::new(&settings)
        .await?;

    let mut guard = state.grpc_connection.lock().await;
    *guard = Some(client);

    Ok("Connected to gRPC server".into())
}

#[tauri::command]
pub async fn get_graph_data(
    state: State<'_, AppState>,
    start_time: u64,
    end_time: u64,
) -> Result<Response, String> {
    let mut client = {
        let guard = state.grpc_connection.lock().await;
        // クローン可能なので clone してガードをすぐ手放す
        guard
            .as_ref()
            .cloned()
            .ok_or_else(|| "gRPC client is not connected".to_string())?
    };
    let start_timestamp = Timestamp {
        seconds: start_time as i64,
        nanos: 0,
    };
    let end_timestamp = Timestamp {
        seconds: end_time as i64,
        nanos: 0,
    };

    let resp = client
        .get_ambient_conditions(tonic::Request::new(GetAmbientConditionsRequest {
            start_time: Some(start_timestamp),
            end_time: Some(end_timestamp),
            samples: Some(1000),
        }))
        .await
        .map_err(|e| e.to_string())?;

    // TODO: gRPCコールによって受け取ったバイナリデータをデコードしたものをまたエンコードしているはずで無駄な処理をしているはず
    // できそうなら、受け取ったバイナリデータをそのままフロントエンドに渡したい
    let binarized_ambient_condition = resp.into_inner().encode_to_vec();

    // tauriでは、Rust側とフロントエンド側とのデータのやり取りは通常jsonのようだが、あえてbincodeを用いてバイナリでやりとりしてみる。
    // 理論上はバイナリを用いたほうがパフォーマンスが良いはずだが、特に体感として違いはなかった。
    // with_fixed_int_encoding() を使わないと、フロントエンド側でデコードできなかった

    Ok(Response::new(binarized_ambient_condition))
}
