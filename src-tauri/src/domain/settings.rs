use serde::Serialize;

/// アプリケーションの設定を表すエンティティ
#[derive(Debug, Clone, Serialize)]
pub struct Settings {
    pub url: String,

    #[serde(rename = "accessToken")]
    pub access_token: String,
}
