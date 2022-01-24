use std::{future::Future, pin::Pin};

use nix::sys::inotify::AddWatchFlags;
use tokio::sync::oneshot::Receiver as OnceRecv;
use tokio_stream::{wrappers::ReceiverStream, Stream};

use crate::{handle::WatchError, WatchEvent};

type WatchResult<T> = Result<T, WatchError>;

pub struct FileWatchFuture(OnceRecv<WatchEvent>);
pub struct FileWatchStream(ReceiverStream<WatchEvent>);
pub struct DirectoryWatchFuture(OnceRecv<WatchEvent>);
pub struct DirectoryWatchStream(ReceiverStream<WatchEvent>);

impl Future for FileWatchFuture {
    type Output = WatchResult<AddWatchFlags>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx).map(|it| match it {
            Ok(e) => Ok(e.flags),
            Err(_) => Err(WatchError::WatcherShutdown),
        })
    }
}

impl Future for DirectoryWatchFuture {
    type Output = WatchResult<WatchEvent>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Pin::new(&mut self.0)
            .poll(cx)
            .map(|it| it.map_err(|_| WatchError::WatcherShutdown))
    }
}

impl Stream for FileWatchStream {
    type Item = AddWatchFlags;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        Pin::new(&mut self.0)
            .poll_next(cx)
            .map(|ready| ready.map(|event| event.flags))
    }
}

impl Stream for DirectoryWatchStream {
    type Item = WatchEvent;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx)
    }
}
