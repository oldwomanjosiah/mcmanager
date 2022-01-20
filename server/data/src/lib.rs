#![allow(dead_code)]
//! Protocol Buffer Data Definitions for mcmanager

pub mod prelude;

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

/// Use to implement `From<Result<T, E>>` for some type
macro_rules! from_result {
    ($type:ident, $ok:ident($okt:ty), $err:ident($errt:ty)) => {
        impl<T: Into<$okt>, E: Into<$errt>> From<Result<T, E>> for $type {
            fn from(from: Result<T, E>) -> Self {
                match from {
                    Ok(ok) => $type::$ok(ok.into()),
                    Err(err) => $type::$err(err.into()),
                }
            }
        }
    };
}

/// Event Subscriptions and System Snapshots
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

pub mod auth {
    mod proto {
        tonic::include_proto!("auth");
    }

    doc_inline! {
        pub use proto::AuthRequest;
        pub use proto::FailureReason;
        pub use proto::Tokens;
        pub use proto::AuthResponse;
        pub use proto::auth_response::Authorization;
        pub use proto::RefreshRequest;

        pub use proto::auth_server::Auth;
    }

    use proto::auth_server::AuthServer;
    server!(AuthServer, Auth);
    from_result!(Authorization, Token(Tokens), Failure(i32));
}
