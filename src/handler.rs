use crate::bounded::{BoxFuture, Send};
use crate::{Request, Result, Wrap};

use std::future::Future;

/// An asynchronous HTTP handler.
#[async_trait::async_trait]
pub trait Handler<'r>: Send + Sync {
    /// Call this handler with the given request.
    async fn call(&self, req: &'r mut Request) -> Result;

    /// Wrap this handler in some middleware.
    fn wrap<W>(self, wrap: W) -> Wrapped<Self, W>
    where
        Self: Sized,
        W: Wrap<'r>,
    {
        Wrapped {
            handler: self,
            wrap,
        }
    }
}

/// A type-erased [`Handler`].
pub type BoxHandler = Box<dyn for<'r> Handler<'r>>;

impl<'r> Handler<'r> for BoxHandler {
    fn call<'a, 'o>(&'a self, req: &'r mut Request) -> BoxFuture<'o, Result>
    where
        'r: 'o,
        'a: 'o,
    {
        Handler::call(&**self, req)
    }
}

#[async_trait::async_trait]
impl<'r, Fut> Handler<'r> for fn(&'r mut Request) -> Fut
where
    Fut: Future<Output = Result> + Send + Sync,
{
    async fn call(&self, req: &'r mut Request) -> Result {
        self(req).await
    }
}

/// A handler wrapped in some middleware.
///
/// See [`Handler::wrap`] for details.
pub struct Wrapped<H, W> {
    handler: H,
    wrap: W,
}

impl<'r, H, W> Handler<'r> for Wrapped<H, W>
where
    H: Handler<'r>,
    W: Wrap<'r>,
{
    fn call<'a, 'o>(&'a self, req: &'r mut Request) -> BoxFuture<'o, Result>
    where
        'r: 'o,
        'a: 'o,
    {
        self.wrap.call(req, &self.handler)
    }
}
