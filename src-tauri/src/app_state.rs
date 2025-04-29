use pb::tempgrpcd::tempgrpcd_client::TempgrpcdClient;
use std::sync::Arc;
use tokio::sync::Mutex;

use tonic::{service::interceptor::InterceptedService, transport::Channel};

use crate::{
    infrastructure::{db::DbPool, grpc_client::AuthInterceptor},
    pb,
};

type MyGrpcClient =
    Arc<Mutex<Option<TempgrpcdClient<InterceptedService<Channel, AuthInterceptor>>>>>;

/// アプリケーション全体で共有する状態
#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub grpc_connection: MyGrpcClient,
}
