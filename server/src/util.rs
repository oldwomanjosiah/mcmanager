use std::pin::Pin;

use futures::Stream;
use prost::Message;
use tokio::sync::watch::Receiver;
use tokio_stream::wrappers::WatchStream;
use tonic::Response;

/// Turn a type into a [`Response`]
pub trait IntoMessage {
    fn as_msg(self) -> Response<Self>
    where
        Self: Sized;
}

pub trait IntoMessageStream<T, E> {
    fn as_msg(self) -> Response<Pin<Box<dyn Stream<Item = Result<T, E>> + Send>>>
    where
        Self: Sized;
}

// Automatically implement for all types which are prost messages (this technically offers this for
// request messages as well)
impl<T> IntoMessage for T
where
    T: Message,
{
    fn as_msg(self) -> Response<Self>
    where
        Self: Sized,
    {
        Response::new(self)
    }
}

impl<S, T, E> IntoMessageStream<T, E> for S
where
    S: Stream<Item = Result<T, E>> + Send + 'static,
    T: Message,
{
    fn as_msg(self) -> Response<Pin<Box<dyn Stream<Item = Result<T, E>> + Send>>>
    where
        Self: Sized,
    {
        Response::new(Box::pin(self))
    }
}

pub trait Collectable<T> {
    type Output: Stream<Item = T>;

    fn collect(self) -> Self::Output;
}

impl<T> Collectable<T> for Receiver<T>
where
    T: Send + Sync + Clone,
    Self: 'static,
{
    type Output = WatchStream<T>;

    fn collect(self) -> Self::Output {
        WatchStream::new(self)
    }
}
