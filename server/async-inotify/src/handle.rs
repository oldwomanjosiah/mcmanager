use nix::sys::inotify::AddWatchFlags;
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    path::PathBuf,
    time::Duration,
};
use thiserror::Error;
use tokio::{
    sync::{mpsc::Sender as MpscSend, oneshot::Sender as OnceSend},
    task::JoinHandle,
};
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    futures::{DirectoryWatchFuture, DirectoryWatchStream, FileWatchFuture, FileWatchStream},
    task::WatchRequestInner,
};

#[derive(Debug, Clone)]
pub struct Handle {
    pub(crate) request_tx: MpscSend<WatchRequestInner>,
}

#[derive(Debug)]
pub struct OwnedHandle {
    pub(crate) inner: Handle,
    pub(crate) shutdown: OnceSend<()>,
    pub(crate) join: JoinHandle<()>,
}

impl OwnedHandle {
    pub const DEFAULT_SHUTDOWN: Duration = Duration::from_secs(2);
    pub const DEFAULT_REQUEST_BUFFER: usize = 32;

    pub async fn shutdown_with(mut self, wait: Duration) {
        let _ = self.shutdown.send(());

        let join = tokio::time::timeout(wait, &mut self.join);

        match join.await {
            Err(_) => self.join.abort(),
            Ok(Err(e)) => {
                if e.is_cancelled() {
                    panic!("The Watch Task was cancelled without consuming the OwnedHandle");
                }

                std::panic::resume_unwind(e.into_panic());
            }
            Ok(Ok(())) => {}
        }
    }

    pub async fn shutdown(self) {
        self.shutdown_with(Self::DEFAULT_SHUTDOWN).await
    }
}

impl Deref for OwnedHandle {
    type Target = Handle;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for OwnedHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("There is no file or directory at the path: {0}")]
    DoesNotExist(PathBuf),
    #[error("The inode at {0} does not have the correct type for this operation")]
    IncorrectType(PathBuf),
}

#[derive(Debug, Error)]
pub enum WatchError {
    #[error("The watcher task was shutdown while before an event was received")]
    WatcherShutdown,
}

impl Handle {
    pub fn file(&mut self, path: PathBuf) -> Result<WatchRequest<'_, FileEvents>, RequestError> {
        if !path.exists() {
            return Err(RequestError::DoesNotExist(path));
        }
        if path.is_dir() {
            return Err(RequestError::IncorrectType(path));
        }

        let buffer = FileEvents::DEFAULT_BUFFER;

        Ok(WatchRequest {
            handle: self,
            path,
            buffer,
            flags: AddWatchFlags::IN_MODIFY,
            _type: Default::default(),
        })
    }

    pub fn dir(
        &mut self,
        path: PathBuf,
    ) -> Result<WatchRequest<'_, DirectoryEvents>, RequestError> {
        if !path.exists() {
            return Err(RequestError::DoesNotExist(path));
        }
        if !path.is_dir() {
            return Err(RequestError::IncorrectType(path));
        }

        let buffer = DirectoryEvents::DEFAULT_BUFFER;

        Ok(WatchRequest {
            handle: self,
            path,
            buffer,
            flags: AddWatchFlags::IN_MODIFY,
            _type: Default::default(),
        })
    }
}

mod sealed {
    pub trait Sealed {}
}

pub trait WatchType: sealed::Sealed {
    const DEFAULT_BUFFER: usize;
}

pub enum FileEvents {}
pub enum DirectoryEvents {}

impl sealed::Sealed for FileEvents {}
impl sealed::Sealed for DirectoryEvents {}

impl WatchType for FileEvents {
    const DEFAULT_BUFFER: usize = 16;
}
impl WatchType for DirectoryEvents {
    const DEFAULT_BUFFER: usize = 32;
}

pub struct WatchRequest<'handle, T: WatchType> {
    handle: &'handle mut Handle,
    path: PathBuf,
    buffer: usize,
    flags: AddWatchFlags,
    _type: PhantomData<T>,
}

impl<'handle> WatchRequest<'handle, FileEvents> {
    pub fn next(self) -> Result<FileWatchFuture, WatchError> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        self.handle
            .request_tx
            .try_send(WatchRequestInner::Once {
                flags: self.flags,
                path: self.path,
                tx,
            })
            .map_err(|_| WatchError::WatcherShutdown)?;

        Ok(FileWatchFuture(rx))
    }

    pub fn buffer(mut self, size: usize) -> Self {
        self.buffer = size;
        self
    }

    pub fn watch(self) -> Result<FileWatchStream, WatchError> {
        let (tx, rx) = tokio::sync::mpsc::channel(self.buffer);

        self.handle
            .request_tx
            .try_send(WatchRequestInner::Stream {
                flags: self.flags,
                path: self.path,
                tx,
            })
            .map_err(|_| WatchError::WatcherShutdown)?;

        Ok(FileWatchStream(ReceiverStream::from(rx)))
    }
}

impl<'handle> WatchRequest<'handle, DirectoryEvents> {
    pub fn next(self) -> Result<DirectoryWatchFuture, WatchError> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        self.handle
            .request_tx
            .try_send(WatchRequestInner::Once {
                flags: self.flags,
                path: self.path,
                tx,
            })
            .map_err(|_| WatchError::WatcherShutdown)?;

        Ok(DirectoryWatchFuture(rx))
    }

    pub fn buffer(mut self, size: usize) -> Self {
        self.buffer = size;
        self
    }

    pub fn watch(self) -> Result<DirectoryWatchStream, WatchError> {
        let (tx, rx) = tokio::sync::mpsc::channel(self.buffer);

        self.handle
            .request_tx
            .try_send(WatchRequestInner::Stream {
                flags: self.flags,
                path: self.path,
                tx,
            })
            .map_err(|_| WatchError::WatcherShutdown)?;

        Ok(DirectoryWatchStream(ReceiverStream::from(rx)))
    }
}
