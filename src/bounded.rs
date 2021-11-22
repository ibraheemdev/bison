use futures_core::Stream;
use std::future::Future;
use std::pin::Pin;

#[cfg(feature = "not-send")]
mod not_send {
    use super::*;

    pub trait Send {}
    impl<T> Send for T {}

    pub trait Sync {}
    impl<T> Sync for T {}

    pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
    pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + 'a>>;
    pub type BoxError = Box<dyn std::error::Error>;
}

#[cfg(not(feature = "not-send"))]
mod send {
    use super::*;

    pub use std::marker::{Send, Sync};

    pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'a>>;
    pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + Sync + 'a>>;
    pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
}

#[cfg(feature = "not-send")]
pub use not_send::*;

#[cfg(not(feature = "not-send"))]
pub use send::*;
