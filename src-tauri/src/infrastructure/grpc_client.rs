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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::settings::Settings;
    use tonic::Request;

    #[test]
    fn auth_interceptor_inserts_header() {
        let token = MetadataValue::from_str("Bearer abc").expect("ok");
        let mut interceptor = AuthInterceptor { token };
        let req = Request::new(());
        let req = interceptor.call(req).expect("call ok");
        let meta = req.metadata();
        assert!(meta.get("authorization").is_some());
        let v = meta.get("authorization").unwrap().to_str().unwrap();
        assert_eq!(v, "Bearer abc");
    }

    #[tokio::test]
    async fn new_rejects_invalid_url() {
        let s = Settings {
            id: 1,
            url: "not-a-url".to_string(),
            access_token: "t".to_string(),
            use_proxies: false,
            proxy_url: "".to_string(),
        };

        let res = new(&s).await;
        assert!(matches!(res, Err(GrpcClientError::InvalidUrl(_))));
    }

    #[tokio::test]
    async fn new_rejects_invalid_auth_token() {
        // using a token containing a NUL should make MetadataValue::from_str fail
        let s = Settings {
            id: 1,
            url: "https://example.com".to_string(),
            access_token: "bad\0token".to_string(),
            use_proxies: false,
            proxy_url: "".to_string(),
        };

        let res = new(&s).await;
        assert!(matches!(res, Err(GrpcClientError::InvalidAuthToken(_))));
    }

    #[tokio::test]
    async fn new_rejects_invalid_proxy_url() {
        let s = Settings {
            id: 1,
            url: "https://example.com".to_string(),
            access_token: "t".to_string(),
            use_proxies: true,
            proxy_url: "not-a-uri".to_string(),
        };

        let res = new(&s).await;
        // proxy parse/connect should fail; accept either InvalidUri or Transport
        assert!(
            matches!(res, Err(GrpcClientError::InvalidUri(_)))
                || matches!(res, Err(GrpcClientError::Transport(_)))
        );
    }
}
