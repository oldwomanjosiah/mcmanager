use inotify_sys as ffi;
use std::{collections::HashMap, ffi::CString, os::raw::c_int, path::PathBuf};
use tokio::sync::oneshot::Sender;

pub struct Inotify {
    fd: c_int,
    watchers: HashMap<PathBuf, WatchState>,
}

struct WatchState {
    wd: c_int,
    watchers: Vec<Sender<String>>,
}

impl Inotify {
    pub fn new() -> Self {
        // SAFETY
        //
        // See Also: https://man7.org/linux/man-pages/man7/inotify.7.html
        let fd = unsafe { ffi::inotify_init1(ffi::IN_NONBLOCK) };
        Self {
            fd,
            watchers: Default::default(),
        }
    }

    pub fn add_watcher(&mut self, path: PathBuf, sender: Sender<String>) {
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
                let cstr_name = CString::new(path.to_str().unwrap()).unwrap();

                let wd = unsafe {
                    inotify_sys::inotify_add_watch(
                        self.fd,
                        cstr_name.as_ptr(),
                        inotify_sys::IN_MODIFY | inotify_sys::IN_ONESHOT,
                    )
                };

                let watchers = Vec::from([sender]);

                self.watchers.insert(path, WatchState { wd, watchers });
            }
        }
    }

    pub fn handle_events(&mut self) {
        for event in iterator::RawEventIter::new(self.fd) {
            let rem = if let Some((path, state)) =
                self.watchers.iter_mut().find(|(_, v)| v.wd == event.wd)
            {
                if state.watchers.len() != 0 {
                    let mut watchers = Vec::new();
                    std::mem::swap(&mut watchers, &mut state.watchers);

                    for watcher in watchers {
                        // We do not care if the receiver was dropped
                        let name = path.display().to_string();

                        assert!(watcher.send(format!("{name}: {event:#?}")).is_ok());
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
        unsafe { ffi::close(self.fd) };
    }
}

impl mio::event::Source for Inotify {
    fn register(
        &mut self,
        registry: &mio::Registry,
        token: mio::Token,
        interests: mio::Interest,
    ) -> std::io::Result<()> {
        mio::unix::SourceFd(&self.fd).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &mio::Registry,
        token: mio::Token,
        interests: mio::Interest,
    ) -> std::io::Result<()> {
        mio::unix::SourceFd(&self.fd).reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &mio::Registry) -> std::io::Result<()> {
        mio::unix::SourceFd(&self.fd).deregister(registry)
    }
}

mod iterator {
    use ffi::inotify_event as RawInotifyEvent;
    use inotify_sys as ffi;
    use std::{
        ffi::{c_void, CStr},
        mem::MaybeUninit,
        os::raw::c_int,
    };

    #[derive(Debug)]
    pub struct RawEvent {
        pub wd: c_int,
        pub mask: u32,
        pub cookie: u32,
        pub name: Option<String>,
    }

    // Event size is defined to be RawEvent + NAME_MAX (for the path component) + 1 (for the null byte)
    const EVENT_SIZE: usize = std::mem::size_of::<RawInotifyEvent>() + 255 + 1;

    const COPY_SIZE: usize = std::mem::size_of::<RawInotifyEvent>();

    #[derive(Debug)]
    #[repr(align(4))] // This was chosen as it was the alignment of RawInotifyEvent, which is a requirement
    pub struct RawEventIter {
        fd: c_int,
        buffer: [u8; EVENT_SIZE],
    }

    impl RawEventIter {
        pub fn new(fd: c_int) -> Self {
            Self {
                fd,
                buffer: [0; EVENT_SIZE],
            }
        }
    }

    impl Iterator for RawEventIter {
        type Item = RawEvent;

        fn next(&mut self) -> Option<Self::Item> {
            let resp = unsafe {
                inotify_sys::read(
                    self.fd,
                    &mut self.buffer as *mut _ as *mut c_void,
                    EVENT_SIZE,
                )
            };

            if resp == -1 {
                return None;
            }
            // debug_assert_eq!(
            //     resp, 1,
            //     "Did not return -1 for no events, should have put one event in buffer"
            // );

            let mut event: MaybeUninit<RawInotifyEvent> = MaybeUninit::uninit();
            event.as_mut_ptr();

            // SAFETY, exactly one instance of this struct was written into the buffer
            // by the call to read
            unsafe {
                std::ptr::copy(
                    self.buffer[0..COPY_SIZE].as_ptr(),
                    event.as_mut_ptr() as *mut u8,
                    COPY_SIZE,
                )
            };

            let RawInotifyEvent {
                wd,
                mask,
                cookie,
                len,
            } = unsafe { event.assume_init() };

            let name = if len > 0 {
                // Null Terminated string present at the end of the event, since event.len > 0
                let name = unsafe {
                    CStr::from_ptr(self.buffer[EVENT_SIZE..].as_ptr() as *const _)
                        .to_string_lossy()
                        .to_string()
                };

                Some(name)
            } else {
                None
            };

            Some(RawEvent {
                wd,
                mask,
                cookie,
                name,
            })
        }
    }
}
