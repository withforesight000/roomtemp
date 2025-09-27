use crate::{
    infrastructure::{crypto::CryptoError, grpc_client::GrpcClientError, keystore::KeystoreError},
    repository::diesel_settings_repository::DieselSettingsRepositoryError,
    usecase::settings::SettingsError,
};

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct UIError {
    message: String,
}

// we must manually implement serde::Serialize
impl serde::Serialize for UIError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.message.as_ref())
    }
}

pub fn url_access_token_empty_error() -> UIError {
    UIError {
        message: "URL or access token is empty".into(),
    }
}

impl From<SettingsError> for UIError {
    fn from(err: SettingsError) -> Self {
        match err {
            SettingsError::DieselSettingsRepository(DieselSettingsRepositoryError::Database(_)) => {
                UIError {
                    message: "Database error occurred".into(),
                }
            }
            SettingsError::DieselSettingsRepository(DieselSettingsRepositoryError::Crypto(e)) => {
                match e {
                    CryptoError::InvalidKey => UIError {
                        message: "Crypto: invalid key detected".into(),
                    },
                    CryptoError::InvalidNonceLen(_) => UIError {
                        message: "Crypto: invalid nonce length detected".into(),
                    },
                    CryptoError::Encrypt => UIError {
                        message: "Crypto: encryption failed".into(),
                    },
                    CryptoError::Decrypt => UIError {
                        message: "Crypto: decryption failed".into(),
                    },
                }
            }
            SettingsError::DieselSettingsRepository(DieselSettingsRepositoryError::Keystore(e)) => {
                match e {
                    KeystoreError::PlatformFailure(_) => UIError {
                        message: "Keystore: platform failure".into(),
                    },
                    KeystoreError::NoStorageAccess(_) => UIError {
                        message: "Keystore: storage unavailable".into(),
                    },
                    KeystoreError::NoEntry => UIError {
                        message: "Keystore: entry not found".into(),
                    },
                    KeystoreError::BadEncoding(_) => UIError {
                        message: "Keystore: invalid encoding".into(),
                    },
                    KeystoreError::BadDataFormat(_, _) => UIError {
                        message: "Keystore: invalid data format".into(),
                    },
                    KeystoreError::TooLong(_, _) => UIError {
                        message: "Keystore: attribute too long".into(),
                    },
                    KeystoreError::Invalid(_, _) => UIError {
                        message: "Keystore: invalid attribute".into(),
                    },
                    KeystoreError::Ambiguous(_) => UIError {
                        message: "Keystore: ambiguous credentials".into(),
                    },
                    KeystoreError::InvalidLength => UIError {
                        message: "Keystore: invalid length detected".into(),
                    },
                    KeystoreError::Other => UIError {
                        message: "Keystore: unknown error occurred".into(),
                    },
                }
            }
        }
    }
}

impl From<r2d2::Error> for UIError {
    fn from(err: r2d2::Error) -> Self {
        UIError {
            message: format!("Database connection error: {}", err),
        }
    }
}

impl From<GrpcClientError> for UIError {
    fn from(err: GrpcClientError) -> Self {
        match err {
            GrpcClientError::InvalidUrl(_) => UIError {
                message: "grpc: invalid URL detected".into(),
            },
            GrpcClientError::InvalidUri(_) => UIError {
                message: "grpc: invalid URI detected".into(),
            },
            GrpcClientError::InvalidTlsDomain(_) => UIError {
                message: "grpc: invalid TLS domain detected".into(),
            },
            GrpcClientError::Transport(_) => UIError {
                message: "grpc: transport error occurred".into(),
            },
            GrpcClientError::InvalidAuthToken(_) => UIError {
                message: "grpc: invalid auth token detected".into(),
            },
        }
    }
}
