use std::{collections::HashMap, ffi::OsString, path::PathBuf};

use nix::{
    errno::Errno,
    sys::inotify::{AddWatchFlags, InitFlags, Inotify, WatchDescriptor},
};
use thiserror::Error;
use tokio::{
    io::unix::{AsyncFd, AsyncFdReadyGuard, AsyncFdReadyMutGuard},
    select,
    sync::mpsc::error::TrySendError,
    sync::mpsc::Receiver as MpscRecv,
    sync::mpsc::Sender as MpscSend,
    sync::oneshot::Receiver as OnceRecv,
    sync::oneshot::Sender as OnceSend,
    task::JoinHandle,
};

use crate::WatchEvent;

#[derive(Debug)]
pub enum WatchRequestInner {
    /// Create a new once watch
    Once {
        path: PathBuf,
        flags: AddWatchFlags,
        tx: OnceSend<WatchEvent>,
    },

    /// Create a new streaming watch
    Stream {
        path: PathBuf,
        flags: AddWatchFlags,
        tx: MpscSend<WatchEvent>,
    },

    /// A watcher was dropped, so we should scan for it and remove it
    Drop,
}

#[derive(Debug)]
pub struct WatcherState {
    instance: AsyncFd<Inotify>,
    request_rx: MpscRecv<WatchRequestInner>,
    shutdown: OnceRecv<()>,
    watches: Watches,
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
            watches: Default::default(),
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
                read_guard = self.instance.readable() => {
                    self.watches.handle_events(read_guard.unwrap()).await.unwrap();
                }

                request = self.request_rx.recv() => {
                    match request {
                        Some(event) => { todo!() },

                        // All senders have been dropped, so there will be no more watches
                        // requested
                        None => break,
                    }
                }

                _ = &mut self.shutdown => {
                    break;
                }
            }
        }
    }
}

#[derive(Debug)]
struct OnceWatcher {
    flags: AddWatchFlags,
    sender: OnceSend<WatchEvent>,
}

#[derive(Debug)]
struct StreamWatcher {
    flags: AddWatchFlags,
    sender: MpscSend<WatchEvent>,
}

#[derive(Debug)]
struct WatchState {
    path: PathBuf,
    once: Vec<OnceWatcher>,
    stream: Vec<StreamWatcher>,
}

#[derive(Debug, Default)]
struct Watches {
    watches: HashMap<WatchDescriptor, WatchState>,
    paths: HashMap<PathBuf, WatchDescriptor>,
}

impl Watches {
    async fn handle_events(
        &mut self,
        mut guard: AsyncFdReadyGuard<'_, Inotify>,
    ) -> Result<(), Errno> {
        let events = guard.get_inner().read_events()?;

        for event in events.into_iter() {
            let flags = event.mask;
            let path = event.name.map(Into::into);

            if let Some(watch) = self.watches.get_mut(&event.wd) {
                let mut replace = Vec::with_capacity(watch.once.len() / 2);

                for once in watch.once.drain(..) {
                    if once.flags.intersects(flags) {
                        let path = path.clone();

                        let _ = once.sender.send(WatchEvent { flags, path });
                    } else {
                        // Events which do not match the current are not removed
                        replace.push(once);
                    }
                }

                std::mem::swap(&mut replace, &mut watch.once);

                for stream in watch.stream.iter_mut() {
                    if stream.flags.intersects(flags) {
                        let path = path.clone();

                        let _ = stream.sender.try_send(WatchEvent { flags, path });
                    }
                }
            }
        }

        guard.clear_ready();
        Ok(())
    }

    async fn handle_request(
        &mut self,
        inotify: &Inotify,
        request: WatchRequestInner,
    ) -> Result<(), Errno> {
        match request {
            WatchRequestInner::Drop => self.clean_watches().await,
            WatchRequestInner::Once { path, flags, tx } => {
                if let Some(wd) = self.paths.get(&path) {
                    let state = self.watches.get_mut(wd).unwrap();
                    state.once.push(OnceWatcher { flags, sender: tx });
                } else {
                    let wd = inotify.add_watch(&path, flags)?;
                    let state = WatchState {
                        path: path.clone(),
                        once: Vec::from([OnceWatcher { flags, sender: tx }]),
                        stream: Default::default(),
                    };

                    self.paths.insert(path, wd);
                    self.watches.insert(wd, state);
                }
            }
            WatchRequestInner::Stream { path, flags, tx } => {
                if let Some(wd) = self.paths.get(&path) {
                    let state = self.watches.get_mut(wd).unwrap();
                    state.stream.push(StreamWatcher { flags, sender: tx });
                } else {
                    let wd = inotify.add_watch(&path, flags)?;
                    let state = WatchState {
                        path: path.clone(),
                        once: Default::default(),
                        stream: Vec::from([StreamWatcher { flags, sender: tx }]),
                    };

                    self.paths.insert(path, wd);
                    self.watches.insert(wd, state);
                }
            }
        }

        Ok(())
    }

    async fn clean_watches(&mut self) {
        todo!("Find and remove unused watches");
    }
}
