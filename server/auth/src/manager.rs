//! Authorization Management

use std::{
    fs::File,
    io::{BufReader, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use futures::{lock::Mutex, TryFutureExt};
use thiserror::Error;
use tracing::{error, warn};

use crate::auth_data::AuthStore;

/// Errors that can occur while interacting with an AuthStore
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Encountered Error While Attempting to Open AuthStore for Reading: {0}")]
    FileReadError(#[source] std::io::Error),

    #[error("Encountered Error While Attempting to Open AuthStore for Writing: {0}")]
    FileWriteError(#[source] std::io::Error),

    #[error("Encountered Error While Attempting to Deserialize AuthStore {0}")]
    DeserializeError(#[from] serde_json::error::Error),
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
}

struct AuthManagerInner {
    config: AuthManagerConfig,
    auth_store: AuthStore,
}

impl AuthManagerInner {
    fn new(config: AuthManagerConfig) -> Result<Self, StoreError> {
        let auth_store = load_store_from_file(&config.users_file)
            .or_else(|e| create_default_store(&config.users_file, e))?;
        Ok(Self { config, auth_store })
    }
}

/// Load an AuthStore from a file
fn load_store_from_file(path: impl AsRef<Path>) -> Result<AuthStore, StoreError> {
    let file = File::open(path.as_ref()).map_err(StoreError::FileReadError)?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).map_err(StoreError::DeserializeError)
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
    serde_json::to_writer(&mut file, &store)?;
    file.flush().map_err(StoreError::FileWriteError)?;

    Ok(store)
}
