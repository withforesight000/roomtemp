use std::str::FromStr;

use tempgrpcd_protos::tempgrpcd::v1::tempgrpcd_service_client::TempgrpcdServiceClient;
use tonic::{
    Request, Status,
    metadata::MetadataValue,
    service::{Interceptor, interceptor::InterceptedService},
    transport::{Channel, ClientTlsConfig},
};
use url::Url;

/// トークンを保持するだけのシンプルな struct
#[derive(Clone)]
pub struct AuthInterceptor {
    token: MetadataValue<tonic::metadata::Ascii>,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        req.metadata_mut()
            .insert("authorization", self.token.clone());
        Ok(req)
    }
}

// TODO: Handle HTTP URL scheme, not just HTTPS
// TODO: Handle cases where authentication is not required
/// Creates a new gRPC client with authentication.
pub async fn new(
    endpoint: &str,
    bearer_token: &str,
) -> Result<
    TempgrpcdServiceClient<InterceptedService<Channel, AuthInterceptor>>,
    Box<dyn std::error::Error>,
> {
    let url = Url::parse(endpoint)?;
    let tls_config = ClientTlsConfig::new()
        .with_enabled_roots()
        .domain_name(url.host_str().unwrap_or(""));

    let channel = Channel::from_shared(endpoint.to_string())
        .map_err(|e| e.to_string())?
        .tls_config(tls_config)
        .map_err(|e| e.to_string())?
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let interceptor = AuthInterceptor {
        token: MetadataValue::from_str(&format!("Bearer {bearer_token}"))
            .map_err(|e| e.to_string())?,
    };
    let client = TempgrpcdServiceClient::with_interceptor(channel, interceptor);

    Ok(client)
}
