use std::{str::FromStr, time::Duration};

use tonic_health::pb::{
    HealthCheckRequest, health_check_response::ServingStatus, health_client::HealthClient,
};

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
    #[error("health check failed")]
    Health(#[source] Status),
    #[error("service is not serving")]
    NotServing,
    #[error("connection check timed out")]
    Timeout,
}

/// トークンを保持するだけのシンプルな struct
#[derive(Clone)]
pub struct AuthInterceptor {
    token: MetadataValue<tonic::metadata::Ascii>,
}

#[cfg(test)]
impl AuthInterceptor {
    /// Builds fixed, non-secret metadata for loopback state tests.
    pub(crate) fn for_test() -> Self {
        Self {
            token: "Bearer test-token".parse().unwrap(),
        }
    }
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
    bounded(new_inner(settings), Duration::from_secs(10)).await
}

/// Applies one deadline to channel creation and the subsequent authenticated check.
async fn bounded<T>(
    future: impl std::future::Future<Output = Result<T, GrpcClientError>>,
    duration: Duration,
) -> Result<T, GrpcClientError> {
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| GrpcClientError::Timeout)?
}

/// Builds and verifies a client using the same channel for health and graph RPCs.
async fn new_inner(
    settings: &Settings,
) -> Result<TempgrpcdServiceClient<InterceptedService<Channel, AuthInterceptor>>, GrpcClientError> {
    let url = Url::parse(&settings.url)?;
    let token = MetadataValue::from_str(&format!("Bearer {}", &settings.access_token))?;
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
        endpoint
            .connect_with_connector(proxy)
            .await
            .map_err(|e| GrpcClientError::Transport(e))?
    } else {
        endpoint.connect().await?
    };

    let interceptor = AuthInterceptor { token };
    verify_health(channel.clone(), interceptor.clone()).await?;
    let client = TempgrpcdServiceClient::with_interceptor(channel, interceptor);

    Ok(client)
}

/// Confirms authenticated availability of the data service without fetching Redis data.
async fn verify_health(
    channel: Channel,
    interceptor: AuthInterceptor,
) -> Result<(), GrpcClientError> {
    let mut client = HealthClient::with_interceptor(channel, interceptor);
    let mut request = Request::new(HealthCheckRequest {
        service: "tempgrpcd.v1.TempgrpcdService".into(),
    });
    request.set_timeout(Duration::from_secs(10));
    let response = client
        .check(request)
        .await
        .map_err(GrpcClientError::Health)?
        .into_inner();
    if response.status != ServingStatus::Serving as i32 {
        return Err(GrpcClientError::NotServing);
    }
    Ok(())
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

#[cfg(test)]
mod health_tests {
    use super::*;
    use tonic::codegen::BoxStream;
    use tonic::{
        Response,
        transport::{Server, server::TcpIncoming},
    };
    use tonic_health::pb::{
        HealthCheckResponse,
        health_server::{Health, HealthServer},
    };

    struct Probe {
        status: Result<i32, tonic::Code>,
        delay: Duration,
    }

    #[tonic::async_trait]
    impl Health for Probe {
        async fn check(
            &self,
            request: Request<HealthCheckRequest>,
        ) -> Result<Response<HealthCheckResponse>, Status> {
            assert_eq!(request.get_ref().service, "tempgrpcd.v1.TempgrpcdService");
            tokio::time::sleep(self.delay).await;
            match self.status {
                Ok(status) => Ok(Response::new(HealthCheckResponse { status })),
                Err(code) => Err(Status::new(code, "private detail")),
            }
        }
        type WatchStream = BoxStream<HealthCheckResponse>;
        async fn watch(
            &self,
            _: Request<HealthCheckRequest>,
        ) -> Result<Response<Self::WatchStream>, Status> {
            Err(Status::unimplemented("unused"))
        }
    }

    async fn run_probe(
        status: Result<i32, tonic::Code>,
        token: &str,
        delay: Duration,
        limit: Duration,
    ) -> Result<(), GrpcClientError> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            Server::builder()
                .add_service(HealthServer::with_interceptor(
                    Probe { status, delay },
                    |request: Request<()>| {
                        if request
                            .metadata()
                            .get("authorization")
                            .and_then(|value| value.to_str().ok())
                            == Some("Bearer correct")
                        {
                            Ok(request)
                        } else {
                            Err(Status::unauthenticated("private detail"))
                        }
                    },
                ))
                .serve_with_incoming_shutdown(TcpIncoming::from(listener), async {
                    let _ = rx.await;
                })
                .await
                .unwrap();
        });
        let channel = Channel::from_shared(format!("http://{address}"))
            .unwrap()
            .connect()
            .await
            .unwrap();
        let result = bounded(
            verify_health(
                channel,
                AuthInterceptor {
                    token: token.parse().unwrap(),
                },
            ),
            limit,
        )
        .await;
        tx.send(()).unwrap();
        server.await.unwrap();
        result
    }

    #[tokio::test]
    async fn health_rpc_checks_authentication_serving_status_and_compatibility() {
        let fast = Duration::ZERO;
        let limit = Duration::from_secs(2);
        assert!(
            run_probe(
                Ok(ServingStatus::Serving as i32),
                "Bearer correct",
                fast,
                limit
            )
            .await
            .is_ok()
        );
        assert!(
            matches!(run_probe(Ok(1), "Bearer wrong", fast, limit).await, Err(GrpcClientError::Health(status)) if status.code() == tonic::Code::Unauthenticated)
        );
        for status in [
            ServingStatus::NotServing as i32,
            ServingStatus::Unknown as i32,
            99,
        ] {
            assert!(matches!(
                run_probe(Ok(status), "Bearer correct", fast, limit).await,
                Err(GrpcClientError::NotServing)
            ));
        }
        for code in [
            tonic::Code::NotFound,
            tonic::Code::Unimplemented,
            tonic::Code::Unavailable,
        ] {
            assert!(
                matches!(run_probe(Err(code), "Bearer correct", fast, limit).await, Err(GrpcClientError::Health(status)) if status.code() == code)
            );
        }
        assert!(matches!(
            run_probe(
                Ok(1),
                "Bearer correct",
                Duration::from_millis(100),
                Duration::from_millis(10)
            )
            .await,
            Err(GrpcClientError::Timeout)
        ));
    }
}

#[cfg(test)]
mod deployed_server_tests {
    use super::*;

    /// Checks the real updated server separately from mock and loopback-service tests.
    #[tokio::test]
    #[ignore = "requires an isolated tempgrpcd instance via ROOMTEMP_TEST_GRPC_ENDPOINT"]
    async fn updated_server_accepts_valid_credentials_and_rejects_invalid_credentials() {
        let endpoint = std::env::var("ROOMTEMP_TEST_GRPC_ENDPOINT").expect("test endpoint");
        let token = std::env::var("ROOMTEMP_TEST_GRPC_TOKEN").expect("isolated test token");
        let channel = Channel::from_shared(endpoint)
            .unwrap()
            .connect()
            .await
            .unwrap();
        let correct = AuthInterceptor {
            token: format!("Bearer {token}").parse().unwrap(),
        };
        verify_health(channel.clone(), correct.clone())
            .await
            .unwrap();
        let wrong = AuthInterceptor {
            token: format!("Bearer {token}-invalid").parse().unwrap(),
        };
        let failure = verify_health(channel.clone(), wrong).await.unwrap_err();
        assert!(
            matches!(failure, GrpcClientError::Health(status) if status.code() == tonic::Code::Unauthenticated)
        );
        let mut client = TempgrpcdServiceClient::with_interceptor(channel, correct);
        let mut request = Request::new(
            tempgrpcd_protos::tempgrpcd::v1::GetAmbientConditionsRequest {
                start_time: Some(pbjson_types::Timestamp {
                    seconds: 0,
                    nanos: 0,
                }),
                end_time: Some(pbjson_types::Timestamp {
                    seconds: 1,
                    nanos: 0,
                }),
                samples: Some(1),
            },
        );
        request.set_timeout(Duration::from_secs(10));
        client.get_ambient_conditions(request).await.unwrap();
    }
}
