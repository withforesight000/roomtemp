use pbjson_types::Timestamp;
use prost::Message;
use tauri::State;
use tauri::ipc::Response;
use tempgrpcd_protos::tempgrpcd::v1::GetAmbientConditionsRequest;

use crate::app_state::AppState;
use crate::controller::settings_controller::SettingsController;
use crate::domain::settings::Settings;
use crate::infrastructure::grpc_client;
use crate::presentation::grpc_error::GrpcError;
use crate::presentation::ui_error::{self, UIError};
use crate::repository::diesel_settings_repository::DieselSettingsRepository;
use std::{future::Future, time::Duration};

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<Settings, UIError> {
    get_settings_inner(&state)
}

/// Reads persisted settings; callers serialize snapshots with connection state.
fn get_settings_inner(state: &AppState) -> Result<Settings, UIError> {
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
    set_settings_inner(&state, url, access_token, use_proxies, proxy_url)
}

/// Saves settings and invalidates the connection atomically with respect to connection snapshots.
fn set_settings_inner(
    state: &AppState,
    url: String,
    access_token: String,
    use_proxies: bool,
    proxy_url: String,
) -> Result<(), UIError> {
    let mut guard = state
        .grpc_connection
        .lock()
        .map_err(|_| ui_error::connection_state_error())?;
    let conn = state.pool.get()?;
    let mut repo = DieselSettingsRepository { conn };
    let mut controller = SettingsController::new(&mut repo);
    controller.set(url, access_token, use_proxies, proxy_url)?;
    guard.invalidate();
    Ok(())
}

#[tauri::command]
pub async fn connect_to_grpc_server(state: State<'_, AppState>) -> Result<String, GrpcError> {
    connect_from_state(&state).await
}

/// Starts a verified connection without exposing the Tauri state wrapper to tests.
async fn connect_from_state(state: &AppState) -> Result<String, GrpcError> {
    connect_with(state, |settings| async move {
        grpc_client::new(&settings).await.map_err(GrpcError::from)
    })
    .await
}

/// Snapshots settings and publishes only the latest verified connection.
async fn connect_with<F, Fut>(state: &AppState, connector: F) -> Result<String, GrpcError>
where
    F: FnOnce(Settings) -> Fut,
    Fut: Future<Output = Result<crate::app_state::GrpcClient, GrpcError>>,
{
    let (generation, settings) = {
        let mut guard = state.grpc_connection.lock().map_err(|_| state_error())?;
        let generation = guard.invalidate();
        let settings = get_settings_inner(state).map_err(|_| {
            GrpcError::new(
                "invalid_configuration",
                "Unable to read connection settings",
            )
        })?;
        (generation, settings)
    };
    if settings.url.is_empty() || settings.access_token.is_empty() {
        return Err(GrpcError::new(
            "invalid_configuration",
            "URL or access token is empty",
        ));
    }
    let result = connector(settings).await;
    let mut guard = state.grpc_connection.lock().map_err(|_| state_error())?;
    if guard.generation != generation {
        return Err(GrpcError::superseded());
    }
    guard.client = Some(result?);
    Ok("Connected to gRPC server".into())
}

/// Reports an unavailable shared-state lock with safe IPC details.
fn state_error() -> GrpcError {
    GrpcError::new("request_failed", "Connection state is unavailable")
}

#[tauri::command]
pub async fn get_graph_data(
    state: State<'_, AppState>,
    start_time: u64,
    end_time: u64,
) -> Result<Response, GrpcError> {
    graph_from_state(&state, start_time, end_time).await
}

/// Retrieves graph data only from a verified client and rejects obsolete results.
async fn graph_from_state(
    state: &AppState,
    start_time: u64,
    end_time: u64,
) -> Result<Response, GrpcError> {
    let (generation, mut client) = {
        let guard = state.grpc_connection.lock().map_err(|_| state_error())?;
        let client = guard.client.clone().ok_or_else(|| {
            GrpcError::new(
                "not_connected",
                "Check the gRPC connection before fetching data",
            )
        })?;
        (guard.generation, client)
    };
    let invalid_time = || GrpcError::new("request_failed", "The selected time range is invalid");
    let mut request = tonic::Request::new(GetAmbientConditionsRequest {
        start_time: Some(
            timestamp_from_epoch_milliseconds(start_time).map_err(|_| invalid_time())?,
        ),
        end_time: Some(timestamp_from_epoch_milliseconds(end_time).map_err(|_| invalid_time())?),
        samples: Some(1000),
    });
    request.set_timeout(Duration::from_secs(10));
    let result = tokio::time::timeout(
        Duration::from_secs(10),
        client.get_ambient_conditions(request),
    )
    .await
    .map_err(|_| GrpcError::timeout())
    .and_then(|result| result.map_err(GrpcError::from));
    let mut guard = state.grpc_connection.lock().map_err(|_| state_error())?;
    if guard.generation != generation {
        return Err(GrpcError::superseded());
    }
    if let Err(error) = &result {
        if error.invalidates_connection() {
            guard.invalidate();
        }
    }
    let bytes = result?.into_inner().encode_to_vec();
    Ok(Response::new(bytes))
}

/// Converts a non-negative Unix epoch millisecond value into a canonical protobuf timestamp.
///
/// The frontend and Tauri command retain their millisecond-based interface. This helper
/// separates whole seconds from the remaining milliseconds and reports values whose seconds
/// cannot be represented by protobuf's signed field instead of wrapping them.
fn timestamp_from_epoch_milliseconds(milliseconds: u64) -> Result<Timestamp, String> {
    const PROTOBUF_MAX_SECONDS: u64 = 253_402_300_799;

    if milliseconds / 1000 > PROTOBUF_MAX_SECONDS {
        return Err("timestamp milliseconds are outside the supported range".to_string());
    }
    let seconds = i64::try_from(milliseconds / 1000)
        .map_err(|_| "timestamp milliseconds are outside the supported range".to_string())?;
    let nanos = i32::try_from((milliseconds % 1000) * 1_000_000)
        .map_err(|_| "timestamp milliseconds are outside the supported range".to_string())?;

    Ok(Timestamp { seconds, nanos })
}

#[cfg(test)]
pub mod __tests {
    use super::*;
    use crate::app_state::AppState;
    use diesel::r2d2::ConnectionManager;
    use diesel::r2d2::Pool;
    use diesel::sqlite::SqliteConnection;
    use std::sync::Arc;
    use std::sync::Mutex;

    // Test helper for calling get_settings without tauri State wrapper
    pub fn get_settings_from_state(state: &AppState) -> Result<Settings, UIError> {
        // Ensure DB schema exists for tests
        crate::infrastructure::db::run_migrations(&state.pool);

        get_settings_inner(state)
    }

    pub fn set_settings_from_state(
        state: &AppState,
        url: &str,
        access_token: &str,
        use_proxies: bool,
        proxy_url: &str,
    ) -> Result<(), UIError> {
        set_settings_inner(
            state,
            url.into(),
            access_token.into(),
            use_proxies,
            proxy_url.into(),
        )
    }

    #[tokio::test]
    async fn get_graph_data_errors_when_not_connected() {
        let manager =
            ConnectionManager::<SqliteConnection>::new("file:memdb_test2?mode=memory&cache=shared");
        let pool = Pool::builder().build(manager).expect("pool");
        let state = AppState {
            pool,
            grpc_connection: Arc::new(Mutex::new(Default::default())),
        };

        let res = graph_from_state(&state, 0, 1).await;
        assert!(res.is_err());
        assert_eq!(res.err().unwrap().code, "not_connected");
    }

    #[test]
    fn converts_epoch_milliseconds_to_canonical_timestamp() {
        let timestamp = timestamp_from_epoch_milliseconds(1_700_000_000_123).unwrap();

        assert_eq!(timestamp.seconds, 1_700_000_000);
        assert_eq!(timestamp.nanos, 123_000_000);
    }

    #[test]
    fn converts_timestamp_boundaries_without_changing_millisecond_precision() {
        assert_eq!(
            timestamp_from_epoch_milliseconds(0).unwrap(),
            Timestamp {
                seconds: 0,
                nanos: 0
            }
        );
        assert_eq!(
            timestamp_from_epoch_milliseconds(999).unwrap(),
            Timestamp {
                seconds: 0,
                nanos: 999_000_000,
            }
        );
        assert_eq!(
            timestamp_from_epoch_milliseconds(1000).unwrap(),
            Timestamp {
                seconds: 1,
                nanos: 0
            }
        );
    }

    #[test]
    fn rejects_milliseconds_that_overflow_timestamp_seconds() {
        assert!(timestamp_from_epoch_milliseconds(u64::MAX).is_err());
    }

    #[test]
    fn get_and_set_settings_roundtrip() {
        let manager =
            ConnectionManager::<SqliteConnection>::new("file:memdb_test3?mode=memory&cache=shared");
        let pool = Pool::builder().build(manager).expect("pool");
        let state = AppState {
            pool,
            grpc_connection: Arc::new(Mutex::new(Default::default())),
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

    #[tokio::test]
    async fn connect_to_grpc_server_rejects_empty_credentials() {
        let manager =
            ConnectionManager::<SqliteConnection>::new("file:memdb_test4?mode=memory&cache=shared");
        let pool = Pool::builder().build(manager).expect("pool");
        let state = AppState {
            pool,
            grpc_connection: Arc::new(Mutex::new(Default::default())),
        };

        // default settings are empty, so connection should error
        crate::infrastructure::db::run_migrations(&state.pool);
        let err = connect_from_state(&state).await;
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
            grpc_connection: Arc::new(Mutex::new(Default::default())),
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
            grpc_connection: Arc::new(Mutex::new(Default::default())),
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

#[cfg(test)]
mod connection_tests {
    use super::*;
    use crate::app_state::{GrpcClient, GrpcConnection};
    use crate::infrastructure::grpc_client::AuthInterceptor;
    use diesel::{
        connection::SimpleConnection,
        r2d2::{ConnectionManager, Pool},
        sqlite::SqliteConnection,
    };
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    use tempgrpcd_protos::tempgrpcd::v1::{
        GetAmbientConditionsResponse,
        tempgrpcd_service_client::TempgrpcdServiceClient,
        tempgrpcd_service_server::{TempgrpcdService, TempgrpcdServiceServer},
    };
    use tonic::{
        Response as GrpcResponse, Status,
        transport::{Channel, Server, server::TcpIncoming},
    };

    /// Creates isolated persisted settings without accessing the OS credential store.
    fn state() -> AppState {
        static ID: AtomicUsize = AtomicUsize::new(0);
        crate::infrastructure::keystore::KeyStore::set_test_key([9; 32]);
        let manager = ConnectionManager::<SqliteConnection>::new(format!(
            "file:connection_test_{}?mode=memory&cache=shared",
            ID.fetch_add(1, Ordering::Relaxed),
        ));
        let pool = Pool::builder().build(manager).unwrap();
        crate::infrastructure::db::run_migrations(&pool);
        let state = AppState {
            pool,
            grpc_connection: Arc::new(Mutex::new(GrpcConnection::default())),
        };
        get_settings_inner(&state).unwrap();
        set_settings_inner(
            &state,
            "https://test.example".into(),
            "test-token".into(),
            false,
            "".into(),
        )
        .unwrap();
        state
    }

    /// Creates a client without opening a socket, for registration race tests.
    fn client() -> GrpcClient {
        TempgrpcdServiceClient::with_interceptor(
            Channel::from_static("http://127.0.0.1:1").connect_lazy(),
            AuthInterceptor::for_test(),
        )
    }

    #[tokio::test]
    async fn latest_connection_wins_and_failed_reconnect_removes_old_client() {
        let state = state();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (finish_tx, finish_rx) = tokio::sync::oneshot::channel();
        let old_state = state.clone();
        let old = tokio::spawn(async move {
            connect_with(&old_state, |_| async {
                started_tx.send(()).unwrap();
                finish_rx.await.unwrap();
                Ok(client())
            })
            .await
        });
        started_rx.await.unwrap();
        connect_with(&state, |_| async { Ok(client()) })
            .await
            .unwrap();
        let generation = state.grpc_connection.lock().unwrap().generation;
        finish_tx.send(()).unwrap();
        assert_eq!(old.await.unwrap().unwrap_err().code, "superseded");
        assert_eq!(state.grpc_connection.lock().unwrap().generation, generation);
        assert!(state.grpc_connection.lock().unwrap().client.is_some());

        let error = connect_with(&state, |_| async {
            Err(GrpcError::new(
                "authentication_failed",
                "Authentication failed",
            ))
        })
        .await
        .unwrap_err();
        assert_eq!(error.code, "authentication_failed");
        assert!(state.grpc_connection.lock().unwrap().client.is_none());
    }

    #[tokio::test]
    async fn successful_save_supersedes_connect_and_failed_save_preserves_connection() {
        let state = state();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (finish_tx, finish_rx) = tokio::sync::oneshot::channel();
        let old_state = state.clone();
        let old = tokio::spawn(async move {
            connect_with(&old_state, |settings| async move {
                assert_eq!(settings.url, "https://test.example");
                started_tx.send(()).unwrap();
                finish_rx.await.unwrap();
                Ok(client())
            })
            .await
        });
        started_rx.await.unwrap();
        set_settings_inner(
            &state,
            "https://new.example".into(),
            "new-token".into(),
            false,
            "".into(),
        )
        .unwrap();
        finish_tx.send(()).unwrap();
        assert_eq!(old.await.unwrap().unwrap_err().code, "superseded");
        assert!(state.grpc_connection.lock().unwrap().client.is_none());
        connect_with(&state, |settings| async move {
            assert_eq!(settings.url, "https://new.example");
            Ok(client())
        })
        .await
        .unwrap();
        let generation = state.grpc_connection.lock().unwrap().generation;
        state.pool.get().unwrap().batch_execute(
            "CREATE TRIGGER reject_save BEFORE INSERT ON settings BEGIN SELECT RAISE(ABORT, 'test failure'); END;"
        ).unwrap();
        assert!(
            set_settings_inner(
                &state,
                "https://failed.example".into(),
                "other".into(),
                false,
                "".into()
            )
            .is_err()
        );
        let guard = state.grpc_connection.lock().unwrap();
        assert_eq!(guard.generation, generation);
        assert!(guard.client.is_some());
        drop(guard);
        assert_eq!(
            get_settings_inner(&state).unwrap().url,
            "https://new.example"
        );
    }

    struct GraphService {
        code: Option<tonic::Code>,
        gate: Option<(Arc<tokio::sync::Notify>, Arc<tokio::sync::Notify>)>,
    }

    #[tonic::async_trait]
    impl TempgrpcdService for GraphService {
        async fn get_ambient_conditions(
            &self,
            request: tonic::Request<GetAmbientConditionsRequest>,
        ) -> Result<GrpcResponse<GetAmbientConditionsResponse>, Status> {
            assert!(request.metadata().contains_key("grpc-timeout"));
            if let Some((started, finish)) = &self.gate {
                started.notify_one();
                finish.notified().await;
            }
            if let Some(code) = self.code {
                return Err(Status::new(code, "private details"));
            }
            Ok(GrpcResponse::new(GetAmbientConditionsResponse::default()))
        }
    }

    /// Starts a real data RPC service and registers its client for graph state tests.
    async fn graph_server(
        state: &AppState,
        service: GraphService,
    ) -> (
        tokio::sync::oneshot::Sender<()>,
        tokio::task::JoinHandle<()>,
    ) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stop, shutdown) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            Server::builder()
                .add_service(TempgrpcdServiceServer::new(service))
                .serve_with_incoming_shutdown(TcpIncoming::from(listener), async {
                    let _ = shutdown.await;
                })
                .await
                .unwrap();
        });
        let channel = Channel::from_shared(format!("http://{address}"))
            .unwrap()
            .connect()
            .await
            .unwrap();
        state.grpc_connection.lock().unwrap().client = Some(
            TempgrpcdServiceClient::with_interceptor(channel, AuthInterceptor::for_test()),
        );
        (stop, task)
    }

    #[tokio::test]
    async fn graph_errors_invalidate_only_connection_failures() {
        for (code, invalidates) in [
            (tonic::Code::Unauthenticated, true),
            (tonic::Code::Unavailable, true),
            (tonic::Code::DeadlineExceeded, true),
            (tonic::Code::InvalidArgument, false),
            (tonic::Code::ResourceExhausted, false),
            (tonic::Code::Internal, false),
        ] {
            let state = state();
            let (stop, task) = graph_server(
                &state,
                GraphService {
                    code: Some(code),
                    gate: None,
                },
            )
            .await;
            let error = graph_from_state(&state, 0, 1000).await.err().unwrap();
            assert_eq!(error.invalidates_connection(), invalidates);
            assert_eq!(
                state.grpc_connection.lock().unwrap().client.is_none(),
                invalidates
            );
            stop.send(()).unwrap();
            task.await.unwrap();
        }
    }

    #[tokio::test]
    async fn graph_success_and_failure_from_old_generation_are_discarded() {
        for code in [None, Some(tonic::Code::Unauthenticated)] {
            let state = state();
            let started = Arc::new(tokio::sync::Notify::new());
            let finish = Arc::new(tokio::sync::Notify::new());
            let (stop, task) = graph_server(
                &state,
                GraphService {
                    code,
                    gate: Some((started.clone(), finish.clone())),
                },
            )
            .await;
            let old_state = state.clone();
            let request = tokio::spawn(async move { graph_from_state(&old_state, 0, 1000).await });
            started.notified().await;
            connect_with(&state, |_| async { Ok(client()) })
                .await
                .unwrap();
            finish.notify_one();
            assert_eq!(request.await.unwrap().err().unwrap().code, "superseded");
            assert!(state.grpc_connection.lock().unwrap().client.is_some());
            stop.send(()).unwrap();
            task.await.unwrap();
        }
    }
}
