//! Protocol Buffer Data Definitions for mcmanager

mod prelude;
mod util;

macro_rules! doc_inline {
    () => {};
    (
        $( #[$meta:meta] )* $vis:vis use $path:path as $ident:ident; $($tt:tt)*
    ) => {
        #[doc(inline)]
        $(#[$meta])*
        $vis use $path as $ident;

        doc_inline!($($tt)*);
    };
    (
        $( #[$meta:meta] )* $vis:vis use $path:path; $($tt:tt)*
    ) => {
        #[doc(inline)]
        $(#[$meta])*
        $vis use $path;

        doc_inline!($($tt)*);
    };
}

pub trait IntoServer<T> {
    fn into_server(self) -> T;
}

/// Implement Extensions on some Service / Server Pair for quick conversions
macro_rules! server {
    ($server:ident, $service:ident) => {
        impl<T> $crate::IntoServer<$server<T>> for T
        where
            T: $service + Sized + 'static,
        {
            fn into_server(self) -> $server<T> {
                $server::new(self).accept_gzip().send_gzip()
            }
        }
    };
}

pub mod events {

    mod proto {
        tonic::include_proto!("event");
    }

    // Re-Exports
    doc_inline! {
        /// Serve an Events Service
        pub use proto::Event as EventResponse;
        pub use proto::SystemSnapshot;
        pub use proto::EventSubscription;
        pub use proto::event::Event;
        pub use proto::events_server::Events;
    }

    use proto::events_server::EventsServer;
    server!(EventsServer, Events);
}

pub mod hello {
    mod proto {
        tonic::include_proto!("helloworld");
    }

    doc_inline! {
        pub use proto::HelloRequest;
        pub use proto::HelloResponse;
        pub use proto::hello_world_service_server::HelloWorldService as HelloWorld;
    }

    use proto::hello_world_service_server::HelloWorldServiceServer as HelloWorldServer;
    server!(HelloWorldServer, HelloWorld);
}
