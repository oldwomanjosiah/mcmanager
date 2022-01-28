extern crate nix;
extern crate thiserror;
extern crate tokio;
extern crate tokio_stream;

use handle::{Handle, OwnedHandle};
use task::InitError;

pub mod futures;
pub mod handle;
mod task;
#[macro_use]
mod tracing;

// TODO(josiah) convert this to a builder style to allow for request buffer configurations, as well
// as max watchers
pub fn new() -> Result<OwnedHandle, InitError> {
    let (request_tx, request_rx) = tokio::sync::mpsc::channel(OwnedHandle::DEFAULT_REQUEST_BUFFER);
    let inner = Handle { request_tx };
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let join = task::WatcherState::launch(Box::new(task::WatcherState::new(
        request_rx,
        shutdown_rx,
        None,
    )?));

    Ok(OwnedHandle {
        inner,
        join,
        shutdown: shutdown_tx,
    })
}

#[cfg(test)]
mod test {
    use std::{future::Future, io::Write, path::PathBuf, time::Duration};

    use tempdir::TempDir;
    use tokio::{test, time::Timeout};
    use tokio_stream::StreamExt;

    use crate::futures::FileWatchEvent;

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

        let fut = timeout(owner.file(file_path).unwrap().modify(true).next().unwrap());

        wait().await;

        file.change();

        let event = fut.await.unwrap().unwrap();

        eprintln!("{event}");

        assert_eq!(event, FileWatchEvent::Write);
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

        let mut stream = owner.file(file_path).unwrap().modify(true).watch().unwrap();

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

        let mut stream = owner
            .dir(test_dir.path().into())
            .unwrap()
            .modify(true)
            .watch()
            .unwrap();

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

            match item.inner_path.as_deref() {
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
