#[cfg_attr(test, macro_use)]
extern crate tokio;

use std::path::PathBuf;

use handle::{Handle, OwnedHandle};
use nix::sys::inotify::AddWatchFlags;
use task::InitError;
use tokio::sync::{
    mpsc::Sender as MpscSend, oneshot::Receiver as OnceRecv, oneshot::Sender as OnceSend,
};

pub mod futures;
pub mod handle;
mod task;

#[derive(Debug, Clone)]
struct WatchEvent {
    pub flags: AddWatchFlags,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct Watcher {
    handle_id: usize,
    watch_id: usize,
}

struct OnceWatcher {
    flags: AddWatchFlags,
    tx: OnceSend<AddWatchFlags>,
}

struct StreamWatcher {
    flags: AddWatchFlags,
    tx: MpscSend<AddWatchFlags>,
}

// TODO(josiah) convert this to a builder style to allow for request buffer configurations, as well
// as max watchers
pub async fn new() -> Result<OwnedHandle, InitError> {
    let (request_tx, request_rx) = tokio::sync::mpsc::channel(OwnedHandle::DEFAULT_REQUEST_BUFFER);
    let inner = Handle { request_tx };
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let join = task::WatcherState::new(request_rx, shutdown_rx)?.launch();

    Ok(OwnedHandle {
        inner,
        join,
        shutdown: shutdown_tx,
    })
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use tokio::test;

    #[test]
    async fn it_works() -> Result<(), Box<dyn Error>> {
        assert!(true, "Hello, Test");

        Ok(())
    }
}
