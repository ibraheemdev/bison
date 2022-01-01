//! Asynchronous functions that can handle HTTP requests.

mod context;
mod erased;
mod extract;
mod function;
mod wrapped;

pub use context::Context;
pub(crate) use erased::{erase, Erased};
pub(crate) use extract::Extract;
pub use wrapped::Wrapped;

use crate::bounded::{Send, Sync};
use crate::reject::IntoRejection;
use crate::{Respond, Wrap};

/// An asynchronous HTTP handler.
///
/// You should not need to interact this trait directly, it is automatically
/// implemented for handler functions.
#[crate::async_trait_internal]
pub trait Handler<C>: Send + Sync + 'static {
    /// The handler's response.
    type Response: Respond;

    /// An error that can occur when calling the handler.
    type Rejection: IntoRejection;

    /// Call the handler with some context about the request.
    async fn call(&self, cx: C) -> Result<Self::Response, Self::Rejection>;

    /// Wrap a handler with some middleware.
    fn wrap<W>(self, wrap: W) -> Wrapped<Self, C, W>
    where
        W: Wrap,
        Self: Sized,
    {
        Wrapped::new(self, wrap)
    }
}
