use serde::Serialize;

/// アプリケーションの設定を表すエンティティ
#[derive(Debug, Clone, Serialize)]
pub struct Settings {
    pub id: i32,
    pub url: String,

    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "useProxies")]
    pub use_proxies: bool,
    #[serde(rename = "proxyUrl")]
    pub proxy_url: String,
}
