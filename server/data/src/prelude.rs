use std::pin::Pin;

use futures::Stream;
use prost::Message;
use tokio::sync::watch::Receiver;
use tokio_stream::wrappers::WatchStream;
use tonic::{Response, Status};

// Common Type Constructors
pub type StreamResponse<T, E = Status> = std::result::Result<Response<StreamDescriptor<T, E>>, E>;

pub type StreamDescriptor<T, E = Status> =
    Pin<Box<dyn Stream<Item = std::result::Result<T, E>> + Send + 'static>>;

/// Turn a type into a [`Response`]
pub trait IntoMessage {
    fn into_msg(self) -> Response<Self>
    where
        Self: Sized;
}

/// Represents types which can be pinned as a streaming ['Response']
/// See Also: [`Stream`]
pub trait IntoMessageStream<T, E> {
    fn into_msg(self) -> Response<StreamDescriptor<T, E>>
    where
        Self: Sized;
}

// Automatically implement for all types which are prost messages (this technically offers this for
// request messages as well)
impl<T> IntoMessage for T
where
    T: Message,
{
    fn into_msg(self) -> Response<Self>
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
    fn into_msg(self) -> Response<StreamDescriptor<T, E>>
    where
        Self: Sized,
    {
        Response::new(Box::pin(self))
    }
}

/// Extension Trait for types which can be collected as a [`Stream`]
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
