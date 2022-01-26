use std::{
    fmt::{Display, Formatter},
    future::Future,
    pin::Pin,
};

use nix::sys::inotify::AddWatchFlags;
use tokio::sync::oneshot::Receiver as OnceRecv;
use tokio_stream::{wrappers::ReceiverStream, Stream};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileWatchEvent {
    Read,
    Write,
    Open,
    Close { writable: bool },
}

impl TryFrom<AddWatchFlags> for FileWatchEvent {
    type Error = String;

    fn try_from(it: AddWatchFlags) -> Result<Self, Self::Error> {
        use FileWatchEvent::*;
        match it {
            AddWatchFlags::IN_ACCESS => Ok(Read),
            AddWatchFlags::IN_MODIFY => Ok(Write),
            AddWatchFlags::IN_OPEN => Ok(Open),
            AddWatchFlags::IN_CLOSE_NOWRITE => Ok(Close { writable: false }),
            AddWatchFlags::IN_CLOSE_WRITE => Ok(Close { writable: true }),
            otherwise => Err(format!(
                "FileWatchEvent does not cover the bitpattern 0x{otherwise:8X}"
            )),
        }
    }
}

impl Display for FileWatchEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use FileWatchEvent::*;
        match *self {
            Read => write!(f, "read"),
            Write => write!(f, "written"),
            Open => write!(f, "opened"),
            Close { writable } => write!(
                f,
                "closed {}",
                if writable {
                    "for reading"
                } else {
                    "for writing"
                }
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirectoryWatchEvent {
    pub inner_path: Option<String>,
    pub event: FileWatchEvent,
}

impl Display for DirectoryWatchEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(ref inner_path) = self.inner_path {
            write!(f, "{inner_path} was {}", self.event)
        } else {
            write!(f, "a file was {}", self.event)
        }
    }
}

/// Single Event File Watch
pub struct FileWatchFuture(pub(crate) OnceRecv<DirectoryWatchEvent>);
pub struct FileWatchStream(pub(crate) ReceiverStream<DirectoryWatchEvent>);
pub struct DirectoryWatchFuture(pub(crate) OnceRecv<DirectoryWatchEvent>);
pub struct DirectoryWatchStream(pub(crate) ReceiverStream<DirectoryWatchEvent>);

impl Future for FileWatchFuture {
    type Output = Option<FileWatchEvent>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Pin::new(&mut self.0)
            .poll(cx)
            .map(|it| it.ok().map(|event| event.event))
    }
}

impl Future for DirectoryWatchFuture {
    type Output = Option<DirectoryWatchEvent>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx).map(|it| it.ok())
    }
}

impl Stream for FileWatchStream {
    type Item = FileWatchEvent;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        Pin::new(&mut self.0)
            .poll_next(cx)
            .map(|it| it.map(|event| event.event))
    }
}

impl Stream for DirectoryWatchStream {
    // TODO(josiah) update this so that the item type can be WatchResult<WatchEvent>
    type Item = DirectoryWatchEvent;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx)
    }
}
