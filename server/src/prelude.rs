use std::pin::Pin;

use futures::Stream;
use tonic::Response;
use tonic::Status;

// Re-Exports

pub use crate::util::Collectable;
pub use crate::util::IntoMessage;
pub use crate::util::IntoMessageStream;
pub use anyhow::Context;
pub use anyhow::Result;
pub use futures::StreamExt;

// Common Type Constructors

pub type StreamResponse<T, E = Status> = std::result::Result<Response<StreamDescriptor<T, E>>, E>;

pub type StreamDescriptor<T, E = Status> =
    Pin<Box<dyn Stream<Item = std::result::Result<T, E>> + Send + 'static>>;
