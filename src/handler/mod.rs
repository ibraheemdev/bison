mod erased;
mod extract;
mod function;
mod wrapped;

pub use erased::{BoxReturn, Erased};
pub use extract::Extract;
pub use wrapped::Wrapped;

use crate::bounded::{Send, Sync};
use crate::error::IntoResponseError;
use crate::{Responder, WithContext, Wrap};

use std::future::Future;

/// An asynchronous HTTP handler.
///
/// You should not need to interact this trait directly, it is automatically
/// implemented for handler functions.
pub trait Handler<'a, C>: Send + Sync
where
    C: WithContext<'a>,
{
    /// The handler's response.
    type Response: Responder;

    /// An error that can occur when calling the handler.
    type Error: IntoResponseError;

    /// The future returned by [`call`](Self::call).
    type Future: Future<Output = Result<Self::Response, Self::Error>> + Send + 'a;

    /// Call the handler with some context about the request.
    fn call(&'a self, cx: C::Context) -> Self::Future;

    /// Wrap a handler with some middleware.
    fn wrap<W>(self, wrap: W) -> Wrapped<Self, C, W>
    where
        W: Wrap,
        Self: Sized,
    {
        Wrapped::new(self, wrap)
    }
}
