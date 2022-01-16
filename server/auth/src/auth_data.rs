use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthLevel {
    Admin,
    ReadOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthorization {
    pub username: String,
    pub password: String,
    pub auth_level: AuthLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStore {
    pub users: Vec<UserAuthorization>,
}

impl AuthStore {
    /// Create a Default User Store to be written to a file
    pub fn default_store() -> Self {
        Self {
            users: Vec::from([UserAuthorization {
                username: "admin".into(),
                password: "mcmanager".into(),
                auth_level: AuthLevel::Admin,
            }]),
        }
    }
}

pub mod tokens {
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use thiserror::Error;

    /// Represents a part of a token that can be url encoded and decoded
    ///
    /// [See Also](https://jwt.io/introduction)
    pub trait TokenPart: Serialize + DeserializeOwned {
        fn encode(&self) -> Result<String, Error> {
            Ok(base64::encode_config(
                serde_json::to_string(self)?,
                base64::URL_SAFE_NO_PAD,
            ))
        }

        fn decode(from: &str) -> Result<Self, Error> {
            let decoded = base64::decode_config(from, base64::URL_SAFE_NO_PAD)?;
            serde_json::from_slice(&decoded).map_err(Into::into)
        }
    }

    #[derive(Debug, Error)]
    pub enum Error {
        #[error("Error while attempting to decode a token from base64: {0}")]
        Base64Encoding(#[from] base64::DecodeError),

        #[error("Error while attempting to deserialize a token: {0}")]
        JsonFormat(#[from] serde_json::error::Error),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AuthToken {
        pub username: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RefreshToken {}

    impl TokenPart for AuthToken {}
    impl TokenPart for RefreshToken {}
}
