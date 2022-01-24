#[cfg_attr(test, macro_use)]
extern crate tokio;

use std::path::PathBuf;

use handle::{Handle, OwnedHandle};
use nix::sys::inotify::AddWatchFlags;
use task::InitError;

pub mod futures;
pub mod handle;
mod task;

#[derive(Debug, Clone)]
pub struct WatchEvent {
    pub flags: AddWatchFlags,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct Watcher {
    handle_id: usize,
    watch_id: usize,
}

// TODO(josiah) convert this to a builder style to allow for request buffer configurations, as well
// as max watchers
pub fn new() -> Result<OwnedHandle, InitError> {
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
    use std::{error::Error, future::Future, io::Write, path::PathBuf, time::Duration};

    use anyhow::Result;
    use nix::sys::inotify::AddWatchFlags;
    use tempdir::TempDir;
    use tokio::{
        test,
        time::{error::Elapsed, Timeout},
    };
    use tokio_stream::StreamExt;

    fn setup_testdir() -> TempDir {
        TempDir::new("testdir").unwrap()
    }

    struct TestFile(PathBuf, usize);

    fn timeout<F: Future>(fut: F) -> Timeout<F> {
        tokio::time::timeout(Duration::from_secs(2), fut)
    }

    async fn wait() {
        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    fn print_event(event: AddWatchFlags) -> AddWatchFlags {
        match event {
            AddWatchFlags::IN_OPEN => eprintln!("IN_OPEN"),
            AddWatchFlags::IN_ACCESS => eprintln!("IN_ACCESS"),
            AddWatchFlags::IN_MODIFY => eprintln!("IN_MODIFY"),
            AddWatchFlags::IN_ATTRIB => eprintln!("IN_ATTRIB"),
            AddWatchFlags::IN_CLOSE_WRITE => eprintln!("IN_CLOSE_WRITE"),
            AddWatchFlags::IN_CLOSE_NOWRITE => eprintln!("IN_CLOSE_NOWRITE"),
            AddWatchFlags::IN_MOVED_FROM => eprintln!("IN_MOVED_FROM"),
            AddWatchFlags::IN_MOVED_TO => eprintln!("IN_MOVED_TO"),
            AddWatchFlags::IN_CREATE => eprintln!("IN_CREATE"),
            AddWatchFlags::IN_DELETE => eprintln!("IN_DELETE"),
            AddWatchFlags::IN_DELETE_SELF => eprintln!("IN_DELETE_SELF"),
            AddWatchFlags::IN_MOVE_SELF => eprintln!("IN_MOVE_SELF"),
            AddWatchFlags::IN_UNMOUNT => eprintln!("IN_UNMOUNT"),
            AddWatchFlags::IN_Q_OVERFLOW => eprintln!("IN_Q_OVERFLOW"),
            AddWatchFlags::IN_IGNORED => eprintln!("IN_IGNORED"),
            AddWatchFlags::IN_CLOSE => eprintln!("IN_CLOSE"),
            AddWatchFlags::IN_MOVE => eprintln!("IN_MOVE"),
            AddWatchFlags::IN_ONLYDIR => eprintln!("IN_ONLYDIR"),
            AddWatchFlags::IN_DONT_FOLLOW => eprintln!("IN_DONT_FOLLOW"),
            AddWatchFlags::IN_ISDIR => eprintln!("IN_ISDIR"),
            AddWatchFlags::IN_ONESHOT => eprintln!("IN_ONESHOT"),
            AddWatchFlags::IN_ALL_EVENTS => eprintln!("IN_ALL_EVENTS"),
            _ => unreachable!(),
        }

        event
    }

    impl TestFile {
        fn new(path: PathBuf) -> Self {
            std::fs::File::create(&path).unwrap();
            Self(path, 0)
        }

        fn change(&mut self) {
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(true)
                .open(&self.0)
                .unwrap();

            write!(&mut file, "{}: {}", self.0.display(), self.1).unwrap();
            file.flush().unwrap();
            drop(file);
            self.1 += 1;
        }
    }

    #[test]
    async fn it_works() {
        let mut owner = crate::new().unwrap();
        let test_dir = setup_testdir();
        let file_path = test_dir.path().join("test.txt");
        let mut file = TestFile::new(file_path.clone());

        let fut = timeout(owner.file(file_path).unwrap().next().unwrap());

        wait().await;

        file.change();

        let event = print_event(fut.await.unwrap().unwrap());

        assert_eq!(event, AddWatchFlags::IN_MODIFY);
    }

    #[test]
    async fn shutdown() {
        let owner = crate::new().unwrap();

        owner.shutdown().await;
    }

    #[test]
    async fn stream_file() {
        let mut owner = crate::new().unwrap();
        let test_dir = setup_testdir();
        let file_path = test_dir.path().join("test.txt");
        let file = TestFile::new(file_path.clone());

        let mut stream = owner.file(file_path).unwrap().watch().unwrap();

        tokio::spawn(async move {
            let mut file = file;

            wait().await;
            file.change();
            wait().await;
            file.change();
            wait().await;
            file.change();

            drop(file);
        });

        let mut count = 0;
        while let Ok(Some(item)) = timeout(stream.next()).await {
            eprintln!("{item:#?}");
            count += 1;
        }

        assert_eq!(3, count, "Did not get the correct number of events");
    }

    #[test]
    async fn dir_events() {
        let mut owner = crate::new().unwrap();
        let test_dir = setup_testdir();

        let fp1 = test_dir.path().join("test1.txt");
        let fp2 = test_dir.path().join("test2.txt");

        let mut f1 = TestFile::new(fp1.clone());
        let mut f2 = TestFile::new(fp2.clone());

        let mut stream = owner.dir(test_dir.path().into()).unwrap().watch().unwrap();

        wait().await;

        tokio::spawn(async move {
            f1.change();
            f2.change();
        });

        let mut count = 0;
        let mut got_1 = false;
        let mut got_2 = false;

        while let Ok(Some(item)) = timeout(stream.next()).await {
            eprintln!("{item:#?}");

            match item.path.as_ref().map(String::as_str) {
                Some("test1.txt") => got_1 = true,
                Some("test2.txt") => got_2 = true,
                Some(f) => panic!("Did not expect event for {f}"),
                None => {
                    panic!("Did not expect to get no path with directory search: got {item:#?}")
                }
            }
            count += 1;
        }

        assert_eq!(count, 2);
        assert!(got_1);
        assert!(got_2);
    }
}
