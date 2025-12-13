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

    let client = grpc_client::new(&settings).await?;

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

    Ok(Response::new(binarized_ambient_condition))
}

#[cfg(test)]
pub mod __tests {
    use super::*;
    use crate::app_state::AppState;
    use diesel::r2d2::ConnectionManager;
    use diesel::r2d2::Pool;
    use diesel::sqlite::SqliteConnection;
    use std::sync::Arc;
    use tokio::sync::Mutex as TokioMutex;

    // Test helper for calling get_settings without tauri State wrapper
    pub fn get_settings_from_state(state: &AppState) -> Result<Settings, UIError> {
        // Ensure DB schema exists for tests
        crate::infrastructure::db::run_migrations(&state.pool);

        let conn = state.pool.get().unwrap();
        let mut repo = DieselSettingsRepository { conn };
        let mut controller = SettingsController::new(&mut repo);
        match controller.get() {
            Ok(settings) => Ok(settings.unwrap()),
            Err(e) => Err(UIError::from(e)),
        }
    }

    pub fn set_settings_from_state(
        state: &AppState,
        url: &str,
        access_token: &str,
        use_proxies: bool,
        proxy_url: &str,
    ) -> Result<(), UIError> {
        let conn = state.pool.get().unwrap();
        let mut repo = DieselSettingsRepository { conn };
        let mut controller = SettingsController::new(&mut repo);
        controller.set(
            url.to_string(),
            access_token.to_string(),
            use_proxies,
            proxy_url.to_string(),
        )?;
        Ok(())
    }

    pub async fn test_get_graph_data_from_state(
        state: &AppState,
        _start_time: u64,
        _end_time: u64,
    ) -> Result<Response, String> {
        let client = {
            let guard = state.grpc_connection.lock().await;
            guard
                .as_ref()
                .cloned()
                .ok_or_else(|| "gRPC client is not connected".to_string())?
        };
        // Not actually used further for this negative test
        let _ = client;
        Err("not implemented".to_string())
    }

    #[tokio::test]
    async fn get_graph_data_errors_when_not_connected() {
        let manager =
            ConnectionManager::<SqliteConnection>::new("file:memdb_test2?mode=memory&cache=shared");
        let pool = Pool::builder().build(manager).expect("pool");
        let state = AppState {
            pool,
            grpc_connection: Arc::new(TokioMutex::new(None)),
        };

        let res = test_get_graph_data_from_state(&state, 0, 1).await;
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap(),
            "gRPC client is not connected".to_string()
        );
    }

    #[test]
    fn get_and_set_settings_roundtrip() {
        let manager =
            ConnectionManager::<SqliteConnection>::new("file:memdb_test3?mode=memory&cache=shared");
        let pool = Pool::builder().build(manager).expect("pool");
        let state = AppState {
            pool,
            grpc_connection: Arc::new(TokioMutex::new(None)),
        };

        // Initially, get should insert defaults
        let got = get_settings_from_state(&state).expect("get ok");
        assert_eq!(got.id, 1);

        // Now set settings and read back
        set_settings_from_state(&state, "https://x", "tok", true, "http://p").expect("set ok");
        let got2 = get_settings_from_state(&state).expect("get ok");
        assert_eq!(got2.url, "https://x");
        assert_eq!(got2.access_token, "tok");
    }

    pub async fn connect_to_grpc_server_from_state(state: &AppState) -> Result<String, UIError> {
        let settings = get_settings_from_state(state)?;

        if settings.url.is_empty() || settings.access_token.is_empty() {
            return Err(ui_error::url_access_token_empty_error());
        }

        let client = grpc_client::new(&settings).await?;

        let mut guard = state.grpc_connection.lock().await;
        *guard = Some(client);

        Ok("Connected to gRPC server".into())
    }

    #[tokio::test]
    async fn connect_to_grpc_server_rejects_empty_credentials() {
        let manager =
            ConnectionManager::<SqliteConnection>::new("file:memdb_test4?mode=memory&cache=shared");
        let pool = Pool::builder().build(manager).expect("pool");
        let state = AppState {
            pool,
            grpc_connection: Arc::new(TokioMutex::new(None)),
        };

        // default settings are empty, so connection should error
        let err = connect_to_grpc_server_from_state(&state).await;
        assert!(err.is_err());
    }

    // Helper to construct a `tauri::State<AppState>` from a plain reference for tests.
    pub fn make_state_ref(state: &AppState) -> State<'_, AppState> {
        // Safe in tests: `State<'_, T>` is a thin wrapper around `&T` and has the same representation.
        unsafe { std::mem::transmute::<&AppState, State<'_, AppState>>(state) }
    }

    #[test]
    fn get_settings_command_wrapper_works() {
        let manager =
            ConnectionManager::<SqliteConnection>::new("file:memdb_test5?mode=memory&cache=shared");
        let pool = Pool::builder().build(manager).expect("pool");
        let state = AppState {
            pool,
            grpc_connection: Arc::new(TokioMutex::new(None)),
        };

        // Ensure DB schema exists
        crate::infrastructure::db::run_migrations(&state.pool);

        // Should insert defaults and return a Settings
        let s = get_settings(make_state_ref(&state)).expect("get settings");
        assert_eq!(s.id, 1);
    }

    #[test]
    fn set_settings_command_wrapper_works() {
        let manager =
            ConnectionManager::<SqliteConnection>::new("file:memdb_test6?mode=memory&cache=shared");
        let pool = Pool::builder().build(manager).expect("pool");
        let state = AppState {
            pool,
            grpc_connection: Arc::new(TokioMutex::new(None)),
        };

        // Ensure DB schema exists
        crate::infrastructure::db::run_migrations(&state.pool);

        // set should succeed
        set_settings(
            make_state_ref(&state),
            "https://y".into(),
            "tok2".into(),
            false,
            "".into(),
        )
        .expect("set ok");

        let s = get_settings(make_state_ref(&state)).expect("get settings");
        assert_eq!(s.url, "https://y");
    }
}
