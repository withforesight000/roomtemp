use std::str::FromStr;

use http::uri::InvalidUri;
use hyper_http_proxy::{Intercept, Proxy, ProxyConnector};
use hyper_util::client::legacy::connect::HttpConnector;
use rustls::pki_types::InvalidDnsNameError;
use tempgrpcd_protos::tempgrpcd::v1::tempgrpcd_service_client::TempgrpcdServiceClient;
use tonic::{
    Request, Status,
    metadata::{MetadataValue, errors::InvalidMetadataValue},
    service::{Interceptor, interceptor::InterceptedService},
    transport::{Channel, ClientTlsConfig, Error as TransportError},
};
use url::Url;

use crate::domain::settings::Settings;

#[derive(Debug, thiserror::Error)]
pub enum GrpcClientError {
    #[error("failed to parse endpoint URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("invalid URI: {0}")]
    InvalidUri(#[from] InvalidUri),
    #[error("invalid TLS domain name: {0}")]
    InvalidTlsDomain(#[from] InvalidDnsNameError),
    #[error("gRPC transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("invalid authorization token: {0}")]
    InvalidAuthToken(#[from] InvalidMetadataValue),
}

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
// TODO: Handle non-HTTP proxy
/// Creates a new gRPC client with authentication.
pub async fn new(
    settings: &Settings,
) -> Result<TempgrpcdServiceClient<InterceptedService<Channel, AuthInterceptor>>, GrpcClientError> {
    let url = Url::parse(&settings.url)?;
    let tls_config = ClientTlsConfig::new()
        .with_enabled_roots()
        .domain_name(url.host_str().unwrap_or(""));

    let endpoint = Channel::from_shared(url.to_string())?.tls_config(tls_config)?;

    let channel = if settings.use_proxies {
        let proxy = {
            let proxy_uri = settings.proxy_url.parse()?;
            let mut proxy = Proxy::new(Intercept::All, proxy_uri);
            let connector = HttpConnector::new();
            proxy.force_connect();
            ProxyConnector::from_proxy_unsecured(connector, proxy)
        };
        endpoint.connect_with_connector(proxy).await.map_err(|e| {
            eprintln!("Failed to connect via proxy: {e:?}");
            GrpcClientError::Transport(e)
        })?
    } else {
        endpoint.connect().await?
    };

    let interceptor = AuthInterceptor {
        token: MetadataValue::from_str(&format!("Bearer {}", &settings.access_token))?,
    };
    let client = TempgrpcdServiceClient::with_interceptor(channel, interceptor);

    Ok(client)
}
