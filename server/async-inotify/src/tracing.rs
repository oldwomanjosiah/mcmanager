#[macro_export]
#[doc(hidden)]
macro_rules! tracing_unstable {
    ($($tt:tt)*) => {
        #[cfg(all(tokio_unstable, feature = "tracing"))]
        {
            $($tt)*
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! trace {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing_impl::trace!($($tt)*);
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! debug {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing_impl::debug!($($tt)*);
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! info {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing_impl::info!($($tt)*);
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! warn {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing_impl::warn!($($tt)*);
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! error {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing_impl::error!($($tt)*);
    }
}
