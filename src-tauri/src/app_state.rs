use std::sync::{Arc, Mutex};

use tempgrpcd_protos::tempgrpcd::v1::tempgrpcd_service_client::TempgrpcdServiceClient;
use tonic::{service::interceptor::InterceptedService, transport::Channel};

use crate::infrastructure::{db::DbPool, grpc_client::AuthInterceptor};

pub type GrpcClient = TempgrpcdServiceClient<InterceptedService<Channel, AuthInterceptor>>;

/// Connection state guarded together with settings snapshots and successful saves.
/// No network I/O runs while this mutex is held.
#[derive(Default)]
pub struct GrpcConnection {
    pub generation: u64,
    pub client: Option<GrpcClient>,
}

impl GrpcConnection {
    /// Invalidates the client and all work using the previous generation.
    pub fn invalidate(&mut self) -> u64 {
        self.generation += 1;
        self.client = None;
        self.generation
    }
}

/// Shared application state; the connection mutex serializes settings changes.
#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub grpc_connection: Arc<Mutex<GrpcConnection>>,
}
