//! Abstractions over `Send`ness.
//!
//! The traits and type-aliases in this module change
//! depending on whether the `not-send` feature is enabled.

use futures_core::Stream;
use std::error::Error;
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
    pub type BoxError = Box<dyn Error>;

    pub use std::rc::Rc;

    pub use bison_codegen::async_trait_not_send as async_trait;
    pub(crate) use bison_codegen::async_trait_not_send_internal as async_trait_internal;
}

#[cfg(not(feature = "not-send"))]
mod send {
    use super::*;

    pub use std::marker::{Send, Sync};

    /// An dynamically typed [`Future`].
    pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    /// An dynamically typed [`Stream`].
    pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + Sync + 'a>>;

    /// An dynamically typed [`Error`].
    pub type BoxError = Box<dyn Error + Send + Sync>;

    pub use std::sync::Arc as Rc;

    /// A macro for async-trait methods.
    ///
    /// See [`async_trait`](https://docs.rs/async-trait/latest/async_trait/) for details.
    pub use async_trait::async_trait;

    pub(crate) use async_trait::async_trait as async_trait_internal;
}

#[cfg(feature = "not-send")]
pub use not_send::*;

#[cfg(not(feature = "not-send"))]
pub use send::*;
