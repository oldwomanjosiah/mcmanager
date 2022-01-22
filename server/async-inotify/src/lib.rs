extern crate futures;
extern crate inotify_sys;
extern crate mio;
#[cfg_attr(test, macro_use)]
extern crate tokio;

use std::path::PathBuf;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;

use flags::EventMask;
use futures::pin_mut;
use futures::Future;
use futures::FutureExt;
use inotify::Inotify;
use mio::event::Source;
use mio::Events;
use mio::Interest;
use mio::Poll;
use mio::Token;
use mio::Waker;
use std::sync::mpsc::Receiver as MpscReceiver;
use std::sync::mpsc::Sender as MpscSender;
use std::sync::mpsc::TryRecvError;
use tokio::sync::oneshot::error::RecvError;
use tokio::sync::oneshot::Sender as OnceSender;

pub mod flags;
pub mod inotify;

const WAKE_TOKEN: Token = Token(0);
const INOTIFY_TOKEN: Token = Token(1);

#[derive(Clone, Debug)]
pub struct Handle {
    sender: SyncSender<WatchRequest>,
    waker: Arc<Waker>,
}

pub struct OnceFuture {
    channel: tokio::sync::oneshot::Receiver<EventMask>,
}

impl Future for OnceFuture {
    type Output = Result<EventMask, RecvError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.channel.poll_unpin(cx)
    }
}

impl Handle {
    fn request(&self, file: PathBuf) -> OnceFuture {
        let (resp, channel) = tokio::sync::oneshot::channel();

        self.sender.send(WatchRequest { file, resp }).unwrap();

        self.waker.wake().unwrap();

        OnceFuture { channel }
    }
}

#[derive(Debug)]
pub struct WatchRequest {
    file: PathBuf,
    resp: OnceSender<EventMask>,
}

struct State {
    requests: MpscReceiver<WatchRequest>,
    poll: Poll,
    inotify: Inotify,
}

impl State {
    fn new(requests: MpscReceiver<WatchRequest>) -> (Self, Arc<Waker>) {
        let poll = Poll::new().unwrap();
        let waker = Arc::new(Waker::new(poll.registry(), WAKE_TOKEN).unwrap());

        let mut inotify = Inotify::new();

        inotify
            .register(poll.registry(), INOTIFY_TOKEN, Interest::READABLE)
            .unwrap();

        (
            Self {
                requests,
                poll,
                inotify,
            },
            waker,
        )
    }

    fn run(mut self) {
        'main: loop {
            loop {
                eprintln!("Checking requests");
                match self.requests.try_recv() {
                    Ok(event) => self.handle_request(event),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => break 'main,
                }
            }

            let mut events = Events::with_capacity(2);

            eprintln!("Waiting for events to arrive");

            self.poll.poll(&mut events, None).unwrap();

            for event in &events {
                match event.token() {
                    INOTIFY_TOKEN => {
                        eprintln!("Woken up by inotify token");
                        self.inotify.handle_events();
                    }
                    WAKE_TOKEN => {
                        eprintln!("Woken up by wake token");
                        continue 'main;
                    }
                    token => panic!("Unexpected token encountered from watch: {token:?}"),
                }
            }
        }
    }

    fn handle_request(&mut self, event: WatchRequest) {
        eprintln!("Got Event: {event:#?}");
        self.inotify.add_watcher(event.file, event.resp);
    }
}

pub fn spawn() -> Handle {
    let (sender, requests) = std::sync::mpsc::sync_channel(16);

    let (state, waker) = State::new(requests);

    std::thread::spawn(move || state.run());

    Handle { sender, waker }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::time::Duration;
    use tokio::time::sleep;

    use crate::flags::EventFlag;

    use super::*;

    struct CreateFile(PathBuf);

    impl CreateFile {
        fn create(path: PathBuf) -> Self {
            std::fs::File::create(&path).unwrap();
            Self(path)
        }

        fn update_contents(&self) {
            let name = self.0.display().to_string();
            let mut file = std::fs::File::options().write(true).open(&self.0).unwrap();
            write!(&mut file, "Updated: {name}").unwrap();
            file.flush().unwrap();
            drop(file);
        }
    }

    impl Drop for CreateFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[tokio::test]
    async fn spawns() {
        let _ = spawn();
    }

    #[tokio::test]
    async fn gets_event() {
        let handle = spawn();

        let path = PathBuf::from("Hello.txt");

        let file = CreateFile::create(path.clone());

        let fut = handle.request(path);

        sleep(Duration::from_micros(10)).await;

        file.update_contents();

        select! {
            mask = fut => {
                assert_eq!(mask, Ok(EventFlag::Write.into()));
            },
            _ = sleep(Duration::from_secs(2)) => {
                panic!("Inotify future timeout");
            },
        }
    }
}
