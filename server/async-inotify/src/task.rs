use std::{collections::HashMap, ffi::OsString, path::PathBuf, time::Duration};

use nix::{
    errno::Errno,
    sys::inotify::{AddWatchFlags, InitFlags, Inotify, WatchDescriptor},
};
use thiserror::Error;
use tokio::io::Interest;
use tokio::{
    io::unix::{AsyncFd, AsyncFdReadyGuard},
    select,
    sync::mpsc::Receiver as MpscRecv,
    sync::mpsc::{error::TrySendError, Sender as MpscSend},
    sync::oneshot::Receiver as OnceRecv,
    sync::oneshot::Sender as OnceSend,
    task::JoinHandle,
    time::{interval, Interval},
};

use crate::futures::DirectoryWatchEvent;

#[derive(Debug)]
pub(crate) enum WatchRequestInner {
    Start {
        path: PathBuf,
        flags: AddWatchFlags,
        dir: bool,
        sender: Sender,
    },

    /// A watcher was dropped, so we should scan for it and remove it
    #[allow(unused)]
    Drop,
}

#[derive(Debug)]
pub struct WatcherState {
    instance: AsyncFd<Inotify>,
    request_rx: MpscRecv<WatchRequestInner>,
    shutdown: OnceRecv<()>,
    clean_interval: Option<Interval>,
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
    pub(crate) fn new(
        request_rx: MpscRecv<WatchRequestInner>,
        shutdown: OnceRecv<()>,
        clean_duration: Option<Duration>,
    ) -> Result<Self, InitError> {
        let instance =
            AsyncFd::with_interest(Inotify::init(InitFlags::IN_NONBLOCK)?, Interest::READABLE)?;

        let clean_interval = clean_duration.map(|duration| {
            let mut it = interval(duration);
            it.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            it
        });

        Ok(Self {
            instance,
            request_rx,
            shutdown,
            clean_interval,
            watches: Default::default(),
        })
    }

    pub fn launch(self: Box<Self>) -> JoinHandle<()> {
        #[cfg(all(tokio_unstable, feature = "tracing"))]
        {
            tokio::task::Builder::new()
                .name("Inotify Watcher")
                .spawn(self.run())
        }
        #[cfg(not(any(tokio_unstable, feature = "tracing")))]
        {
            tokio::spawn(self.run())
        }
    }

    async fn step(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        async fn maybe(interval: &mut Option<Interval>) {
            match interval {
                Some(interval) => interval.tick().await,
                None => std::future::pending().await,
            };
        }

        select! {
            biased;

            _ = &mut self.shutdown => {
                crate::info!("Shutting Down");

                Ok(false)
            }

            Ok(read_guard) = self.instance.readable() => {
                self.watches
                    .handle_events(read_guard)
                    .await?;

                Ok(true)
            }

            request = self.request_rx.recv() => {
                match request {
                    Some(event) => {
                        self.watches
                            .handle_request(self.instance.get_ref(), event)
                            .await?;

                        Ok(false)
                    }

                    None => {
                        crate::info!("All Handles Dropped, Exiting");

                        Ok(false)
                    }
                }
            }

            _ = maybe(&mut self.clean_interval), if self.watches.dirty => {
                eprintln!("WOKE UP FOR CLEAN");

                Ok(true)
            }
        }
    }

    async fn run(mut self: Box<Self>) {
        if let Some(ref mut tick) = self.clean_interval {
            tick.reset();
        }

        loop {
            match self.step().await {
                Ok(cont) => {
                    if !cont {
                        break;
                    }
                }
                Err(e) => {
                    crate::error!("Got unexpected error in event loop: {e}");
                    break;
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum Sender {
    Once(OnceSend<DirectoryWatchEvent>),
    Stream(MpscSend<DirectoryWatchEvent>),
    None,
}

#[derive(Debug)]
struct SingleWatch {
    flags: AddWatchFlags,
    dir: bool,
    remove: bool,
    sender: Sender,
}

#[derive(Debug)]
struct WatchState {
    path: PathBuf,
    watchers: Vec<SingleWatch>,
}

#[derive(Debug, Default)]
struct Watches {
    watches: HashMap<WatchDescriptor, WatchState>,
    paths: HashMap<PathBuf, WatchDescriptor>,
    pub dirty: bool,
}

impl Watches {
    async fn handle_events(
        &mut self,
        mut guard: AsyncFdReadyGuard<'_, Inotify>,
    ) -> Result<(), Errno> {
        eprintln!("Processing Events from Watches");

        // This should be infallable because we set the FD to non-blocking
        //   and we were woken by the executor with readable
        let events = guard.get_inner().read_events()?;

        for event in events.into_iter() {
            eprintln!("Got Event");
            let flags = event.mask;
            let path = event
                .name
                .map(OsString::into_string)
                .map(Result::ok)
                .flatten();

            if let Some(watch) = self.watches.get_mut(&event.wd) {
                eprintln!(
                    "Got event for path: {} with flags {flags:4X}",
                    watch.path.display()
                );

                let event = flags.try_into();
                if event.is_err() {
                    eprintln!("Got unexpected Flags: 0x{flags:8X}");
                    continue;
                }

                let event = DirectoryWatchEvent {
                    inner_path: path.clone(),
                    event: event.unwrap(),
                };

                for watcher in watch.watchers.iter_mut() {
                    if watcher.remove {
                        continue;
                    }
                    if !watcher.dir && path.is_some() {
                        continue;
                    }

                    if !flags.intersects(watcher.flags) {
                        continue;
                    }

                    // We know that this is an event that they want
                    // So take the sender, send, and replace the sender if necessary

                    let mut replace = std::mem::replace(&mut watcher.sender, Sender::None);

                    replace = match replace {
                        Sender::Once(sender) => {
                            let _ = sender.send(event.clone());

                            watcher.remove = true;
                            self.dirty = true;

                            // send consumes sender, so we cannot defer drop
                            Sender::None
                        }
                        Sender::Stream(sender) => {
                            if let Err(TrySendError::Closed(_)) = sender.try_send(event.clone()) {
                                watcher.remove = true;
                                self.dirty = true;

                                // we defer cleaning up the actual sender
                            }

                            Sender::Stream(sender)
                        }
                        otherwise => otherwise,
                    };

                    std::mem::swap(&mut replace, &mut watcher.sender);
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
            WatchRequestInner::Drop => {
                self.dirty = true;
            }
            WatchRequestInner::Start {
                path,
                flags,
                dir,
                sender,
            } => {
                let watch = SingleWatch {
                    flags,
                    dir,
                    remove: false,
                    sender,
                };

                if let Some(wd) = self.paths.get(&path) {
                    let state = self.watches.get_mut(wd).unwrap();
                    state.watchers.push(watch);
                } else {
                    let wd = inotify.add_watch(&path, flags)?;
                    let state = WatchState {
                        path: path.clone(),
                        watchers: Vec::from([watch]),
                    };

                    self.paths.insert(path, wd);
                    self.watches.insert(wd, state);
                }
            }
        };

        Ok(())
    }

    async fn clean_watches(&mut self) {
        eprintln!("Cleaning Watches");
        todo!("Find and remove unused watches");
    }
}
