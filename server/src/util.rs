use prost::Message;
use tonic::Response;

/// Turn a type into a [`Response`]
pub trait IntoMessage {
    fn as_msg(self) -> Response<Self>
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
