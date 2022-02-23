use futures_core::Stream;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;

pub trait Send {}
impl<T> Send for T {}

pub trait Sync {}
impl<T> Sync for T {}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + 'a>>;
pub type BoxError = Box<dyn Error>;

pub use std::rc::Rc;
