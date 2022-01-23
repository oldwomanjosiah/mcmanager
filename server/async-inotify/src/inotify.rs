use inotify_sys as ffi;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify, WatchDescriptor};
use std::{
    collections::HashMap,
    ffi::CString,
    os::{raw::c_int, unix::prelude::AsRawFd},
    path::PathBuf,
};
use tokio::sync::oneshot::Sender;

use crate::flags::EventMask;

/*
 * This may be able to be replaced by using the nix package abstractions
 */

pub struct Inotify {
    fd: nix::sys::inotify::Inotify,
    watchers: HashMap<PathBuf, WatchState>,
}

struct WatchState {
    wd: WatchDescriptor,
    watchers: Vec<Sender<EventMask>>,
}

impl Inotify {
    pub fn new(blocking: bool) -> Self {
        let flags = if blocking {
            InitFlags::empty()
        } else {
            InitFlags::IN_NONBLOCK
        };

        Self {
            fd: nix::sys::inotify::Inotify::init(flags).unwrap(),
            watchers: Default::default(),
        }
    }

    pub fn add_watcher(&mut self, path: PathBuf, sender: Sender<EventMask>) {
        match self.watchers.get_mut(&path) {
            Some(watch) => {
                eprintln!("Adding to Existing Watch");
                watch.watchers.push(sender);
            }
            None => {
                eprintln!("Creating Watch");
                // Init new watch

                // TODO(josiah) I need to make sure that it is safe to
                // drop a value passed to inotify_add_watch
                let wd = self.fd.add_watch(&path, AddWatchFlags::IN_MODIFY).unwrap();

                let watchers = Vec::from([sender]);

                self.watchers.insert(path, WatchState { wd, watchers });
            }
        }
    }

    pub fn handle_events(&mut self) {
        for event in self.fd.read_events().unwrap() {
            let rem = if let Some((path, state)) =
                self.watchers.iter_mut().find(|(_, v)| v.wd == event.wd)
            {
                if !state.watchers.is_empty() {
                    let mut watchers = Vec::new();
                    std::mem::swap(&mut watchers, &mut state.watchers);

                    for watcher in watchers {
                        // We do not care if the receiver was dropped
                        let name = path.display().to_string();

                        let ret = format!("{name}: {event:#?}");

                        let mask = EventMask(event.mask & EventMask::Any.0);

                        eprintln!("{ret}: Mask: {:4X}", mask.0);

                        assert!(watcher.send(mask).is_ok());
                    }

                    None
                } else {
                    Some(path.clone())
                }
            } else {
                panic!("There were no watchers for the given event: {event:#?}")
            };

            if let Some(rem) = rem {
                let watch_state = self.watchers.get(&rem).unwrap();

                assert_eq!(
                    unsafe { ffi::inotify_rm_watch(self.fd, watch_state.wd) },
                    0,
                    "Error occurred while attempting to remove watch"
                );

                self.watchers.remove(&rem);
            }
        }
    }
}

impl Drop for Inotify {
    fn drop(&mut self) {
        nix::unistd::close(self.fd.as_raw_fd());
    }
}

impl mio::event::Source for Inotify {
    fn register(
        &mut self,
        registry: &mio::Registry,
        token: mio::Token,
        interests: mio::Interest,
    ) -> std::io::Result<()> {
        mio::unix::SourceFd(&self.fd.as_raw_fd()).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &mio::Registry,
        token: mio::Token,
        interests: mio::Interest,
    ) -> std::io::Result<()> {
        mio::unix::SourceFd(&self.fd.as_raw_fd()).reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &mio::Registry) -> std::io::Result<()> {
        mio::unix::SourceFd(&self.fd.as_raw_fd()).deregister(registry)
    }
}
