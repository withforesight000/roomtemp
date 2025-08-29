use std::str::FromStr;

use hyper_http_proxy::{Intercept, Proxy, ProxyConnector};
use hyper_util::client::legacy::connect::HttpConnector;
use tempgrpcd_protos::tempgrpcd::v1::tempgrpcd_service_client::TempgrpcdServiceClient;
use tonic::{
    Request, Status,
    metadata::MetadataValue,
    service::{Interceptor, interceptor::InterceptedService},
    transport::{Channel, ClientTlsConfig},
};
use url::Url;

use crate::domain::settings::Settings;

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
) -> Result<
    TempgrpcdServiceClient<InterceptedService<Channel, AuthInterceptor>>,
    Box<dyn std::error::Error>,
> {
    let url = Url::parse(&settings.url)?;
    let tls_config = ClientTlsConfig::new()
        .with_enabled_roots()
        .domain_name(url.host_str().unwrap_or(""));

    let endpoint = Channel::from_shared(url.to_string())
        .map_err(|e| e.to_string())?
        .tls_config(tls_config)
        .map_err(|e| e.to_string())?;

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
            e.to_string()
        })?
    } else {
        endpoint.connect().await.map_err(|e| e.to_string())?
    };

    let interceptor = AuthInterceptor {
        token: MetadataValue::from_str(&format!("Bearer {}", &settings.access_token))
            .map_err(|e| e.to_string())?,
    };
    let client = TempgrpcdServiceClient::with_interceptor(channel, interceptor);

    Ok(client)
}
