//! Authorization Management

use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};

use data::auth::{FailureReason, Tokens};
use futures::lock::Mutex;
use thiserror::Error;
use tracing::{error, warn};

use crate::auth_data::{
    tokens::{TokenPair, TokenPart},
    AuthStore, User,
};

/// Errors that can occur while interacting with an AuthStore
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Encountered Error While Attempting to Open AuthStore for Reading: {0}")]
    FileReadError(#[source] std::io::Error),

    #[error("Encountered Error While Attempting to Open AuthStore for Writing: {0}")]
    FileWriteError(#[source] std::io::Error),

    #[error("Encountered Error While Attempting to Deserialize AuthStore {0}")]
    DeserializeError(#[from] serde_yaml::Error),
}

/// Configuration for an [`AuthManager`]
pub struct AuthManagerConfig {
    pub users_file: PathBuf,
}

/// Authorization Manager Handle
#[derive(Clone, Debug)]
pub struct AuthManager {
    inner: Arc<Mutex<AuthManagerInner>>,
}

impl AuthManager {
    pub fn new(config: AuthManagerConfig) -> Result<Self, StoreError> {
        let inner = Arc::new(Mutex::new(AuthManagerInner::new(config)?));

        Ok(Self { inner })
    }

    pub fn validate(config: &AuthManagerConfig, creating: bool) -> Result<(), StoreError> {
        match (creating, load_store_from_file(&config.users_file)) {
            (true, Err(e)) => create_default_store(&config.users_file, e),
            (_, l) => l,
        }
        .map(|_| ())
    }

    pub async fn authorize_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Tokens, FailureReason> {
        let mut inner = self.inner.lock().await;

        let user = inner.auth_store.get_username(username);

        if let Some(user) = user {
            let auth = inner
                .auth_store
                .get(&user)
                .expect("User disappeared between get_username and get");

            if auth.password.eq(password) {
                Ok(inner.generate_tokens(&user))
            } else {
                Err(FailureReason::IncorrectPass)
            }
        } else {
            Err(FailureReason::NoUser)
        }
    }

    pub async fn refresh(&self, refresh: &str) -> Result<Tokens, FailureReason> {
        let mut inner = self.inner.lock().await;

        match inner.refresh_store.get(refresh).cloned() {
            Some((user, expiry)) => {
                let now = std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if now < expiry {
                    Ok(inner.generate_tokens(&user))
                } else {
                    inner.refresh_store.remove(refresh);

                    Err(FailureReason::NoToken)
                }
            }
            None => Err(FailureReason::NoToken),
        }
    }
}

struct AuthManagerInner {
    config: AuthManagerConfig,
    auth_store: AuthStore,
    token_store: HashMap<User, (String, u64)>,
    refresh_store: HashMap<String, (User, u64)>,
}

impl AuthManagerInner {
    fn new(config: AuthManagerConfig) -> Result<Self, StoreError> {
        let auth_store = load_store_from_file(&config.users_file)
            .or_else(|e| create_default_store(&config.users_file, e))?;
        Ok(Self {
            config,
            auth_store,
            token_store: Default::default(),
            refresh_store: Default::default(),
        })
    }

    fn generate_tokens(&mut self, user: &User) -> Tokens {
        let pair = TokenPair::for_user(user);

        let access = pair.auth.encode().unwrap();
        let refresh = pair.refresh.encode().unwrap();

        self.token_store
            .insert(user.clone(), (access.clone(), pair.auth.expiry));

        self.refresh_store
            .insert(refresh.clone(), (user.clone(), pair.refresh.expiry));

        let token = Tokens {
            access,
            access_expiry: pair.auth.expiry,
            refresh,
            refresh_expiry: pair.refresh.expiry,
        };

        token
    }
}

/// Load an AuthStore from a file
fn load_store_from_file(path: impl AsRef<Path>) -> Result<AuthStore, StoreError> {
    let file = File::open(path.as_ref()).map_err(StoreError::FileReadError)?;
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).map_err(StoreError::DeserializeError)
}

/// Attempt to recover from an error reading the authstore by creating a default version and
/// writing it to the the expected file
fn create_default_store(
    path: impl AsRef<Path>,
    error: StoreError,
) -> Result<AuthStore, StoreError> {
    warn!("Attempting to recover from: {error} by creating a default store");
    match error {
        StoreError::DeserializeError(e) => {
            error!("Cannot Recover From {e}, we cannot overwrite data");
            return Err(StoreError::DeserializeError(e));
        }
        StoreError::FileReadError(e) if e.kind() != std::io::ErrorKind::NotFound => {
            error!("Cannot Recover From {e}, it is not a missing file");
            return Err(StoreError::FileReadError(e));
        }
        _ => (),
    }

    let store = AuthStore::default_store();

    let mut file = File::create(path).map_err(StoreError::FileWriteError)?;
    serde_yaml::to_writer(&mut file, &store)?;
    file.flush().map_err(StoreError::FileWriteError)?;

    Ok(store)
}
