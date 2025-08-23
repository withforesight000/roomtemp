use std::sync::Arc;

use tempgrpcd_protos::tempgrpcd::v1::tempgrpcd_service_client::TempgrpcdServiceClient;
use tokio::sync::Mutex;
use tonic::{service::interceptor::InterceptedService, transport::Channel};

use crate::infrastructure::{db::DbPool, grpc_client::AuthInterceptor};

type MyGrpcClient =
    Arc<Mutex<Option<TempgrpcdServiceClient<InterceptedService<Channel, AuthInterceptor>>>>>;

/// アプリケーション全体で共有する状態
#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub grpc_connection: MyGrpcClient,
}
