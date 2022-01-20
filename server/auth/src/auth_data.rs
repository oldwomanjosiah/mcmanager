use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Opaque User Handle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct User(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthLevel {
    Admin,
    ReadOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthorization {
    pub password: String,
    pub auth_level: AuthLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStore {
    pub users: HashMap<String, UserAuthorization>,
}

impl AuthStore {
    /// Create a Default User Store to be written to a file
    pub fn default_store() -> Self {
        Self {
            users: HashMap::from([(
                "admin".into(),
                UserAuthorization {
                    password: "mcmanager".into(),
                    auth_level: AuthLevel::Admin,
                },
            )]),
        }
    }

    pub fn get_username(&self, username: &str) -> Option<User> {
        self.users.get(username).map(|_| User(username.into()))
    }

    pub fn get<'s, 'u>(&'s self, user: &'u User) -> Option<&'s UserAuthorization> {
        self.users.get(&user.0)
    }

    pub fn update<'s, 'u>(&'s mut self, user: &'u User, auth: UserAuthorization) -> bool {
        match self.users.get_mut(&user.0) {
            Some(user) => {
                *user = auth;
                true
            }
            None => false,
        }
    }

    /// Create a new user, if it does not exist
    ///
    /// Returns `Err` if the user already exists
    pub fn create(
        &mut self,
        username: String,
        auth: UserAuthorization,
    ) -> Result<&UserAuthorization, ()> {
        match self.users.get(&username) {
            Some(_) => Err(()),
            None => {
                self.users.insert(username.clone(), auth);
                Ok(&self.users[&username])
            }
        }
    }
}

pub mod tokens {
    use std::time::{Duration, UNIX_EPOCH};

    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use thiserror::Error;

    use super::User;

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

        #[error("Error while attempting to serialize or deserialize a token: {0}")]
        JsonFormat(#[from] serde_json::error::Error),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AuthToken {
        pub username: String,
        pub expiry: u64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RefreshToken {
        pub username: String,
        pub expiry: u64,
    }

    impl TokenPart for AuthToken {}
    impl TokenPart for RefreshToken {}

    #[derive(Debug, Clone)]
    pub struct TokenPair {
        pub auth: AuthToken,
        pub refresh: RefreshToken,
    }

    impl TokenPair {
        pub fn for_user(user: &User) -> TokenPair {
            let mins_10 = (std::time::SystemTime::now() + Duration::from_secs(5 * 60))
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let hours_2 = (std::time::SystemTime::now() + Duration::from_secs(120 * 60))
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let auth = AuthToken {
                username: user.0.clone(),
                expiry: mins_10,
            };

            let refresh = RefreshToken {
                username: user.0.clone(),
                expiry: hours_2,
            };

            TokenPair { auth, refresh }
        }
    }
}
