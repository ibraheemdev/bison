use std::error::Error;
use std::future::Future;
use std::pin::Pin;

use futures_core::Stream;

pub use std::marker::{Send, Sync};

/// An dynamically typed [`Future`].
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// An dynamically typed [`Stream`].
pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + Sync + 'a>>;

/// An dynamically typed [`Error`].
pub type BoxError = Box<dyn Error + Send + Sync>;
