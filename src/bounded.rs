mod not_send {
    pub trait Send {}
    impl<T> Send for T {}

    pub trait Sync {}
    impl<T> Sync for T {}

    pub use futures_core::Stream;
    pub use std::error::Error;
    pub use std::future::Future;
    pub use std::rc::Rc;
}

mod send {
    pub trait Future: std::future::Future + Send + Sync {}
    impl<T> Future for T where T: std::future::Future + Send + Sync {}

    pub trait Stream: futures_core::Stream + Send + Sync {}
    impl<T> Stream for T where T: futures_core::Stream + Send + Sync {}

    pub trait Error: std::error::Error + Send + Sync {}
    impl<T> Error for T where T: std::error::Error + Send + Sync {}

    pub use std::marker::{Send, Sync};
    pub use std::sync::Arc as Rc;
}

#[cfg(feature = "not-send")]
pub use not_send::*;

#[cfg(not(feature = "not-send"))]
pub use send::*;
