use std::path::PathBuf;

use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use thiserror::Error;
use tokio::{
    io::unix::AsyncFd, select, sync::mpsc::Receiver as MpscRecv, sync::mpsc::Sender as MpscSend,
    sync::oneshot::Receiver as OnceRecv, sync::oneshot::Sender as OnceSend, task::JoinHandle,
};

#[derive(Debug)]
pub enum WatchRequestInner {
    Once {
        path: PathBuf,
        flags: AddWatchFlags,
        tx: OnceSend<AddWatchFlags>,
    },
    Stream {
        path: PathBuf,
        flags: AddWatchFlags,
        tx: MpscSend<AddWatchFlags>,
    },
}

#[derive(Debug)]
pub struct WatcherState {
    instance: AsyncFd<Inotify>,
    request_rx: MpscRecv<WatchRequestInner>,
    shutdown: OnceRecv<()>,
}

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Could not initalize inotify instance")]
    Inotify(#[from] nix::errno::Errno),

    #[error("Could not register inotify with tokio")]
    AsyncFd(#[from] std::io::Error),
}

impl WatcherState {
    pub fn new(
        request_rx: MpscRecv<WatchRequestInner>,
        shutdown: OnceRecv<()>,
    ) -> Result<Self, InitError> {
        let instance = AsyncFd::new(Inotify::init(InitFlags::IN_NONBLOCK)?)?;

        Ok(Self {
            instance,
            request_rx,
            shutdown,
        })
    }

    pub fn launch(self) -> JoinHandle<()> {
        tokio::task::Builder::new()
            .name("Inotify Watcher")
            .spawn(self.run())
    }

    async fn run(mut self) {
        loop {
            select! {
                read_guard = self.instance.readable_mut() => {
                    todo!("New Events to Dispatch: {read_guard:#?}");
                }

                request = self.request_rx.recv() => {
                    todo!("New Watches to Register: {request:#?}");
                }

                _ = &mut self.shutdown => {
                    break;
                }
            }
        }
    }
}
