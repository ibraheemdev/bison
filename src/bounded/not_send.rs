use futures_core::Stream;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;

pub use once_cell::unsync::OnceCell;
pub use std::cell::{Cell, RefCell};
pub use std::rc::Rc;

pub trait Send {}

impl<T> Send for T {}

pub trait Sync {}

impl<T> Sync for T {}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + 'a>>;
pub type BoxError = Box<dyn Error>;

pub use bison_codegen::async_trait_not_send as async_trait;
pub(crate) use bison_codegen::async_trait_not_send_internal as async_trait_internal;
