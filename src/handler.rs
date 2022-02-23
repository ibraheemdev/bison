use crate::bounded::{BoxFuture, Send};
use crate::{Request, Result, Wrap};

use std::future::Future;

/// An asynchronous HTTP handler.
#[async_trait::async_trait]
pub trait Handler: Send + Sync {
    /// Call this handler with the given request.
    async fn call(&self, req: Request) -> Result;

    /// Wrap this handler in some middleware.
    fn wrap<W>(self, wrap: W) -> Wrapped<Self, W>
    where
        Self: Sized,
        W: Wrap,
    {
        Wrapped {
            handler: self,
            wrap,
        }
    }
}

/// A type-erased [`Handler`].
pub type BoxHandler = Box<dyn Handler>;

impl Handler for BoxHandler {
    fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result>
    where
        'a: 'o,
    {
        Handler::call(&**self, req)
    }
}

#[async_trait::async_trait]
impl<F, Fut> Handler for F
where
    F: Fn(Request) -> Fut + Send + Sync,
    Fut: Future<Output = Result> + Send + Sync,
{
    async fn call(&self, req: Request) -> Result {
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

impl<H, W> Handler for Wrapped<H, W>
where
    H: Handler,
    W: Wrap,
{
    fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result>
    where
        'a: 'o,
    {
        self.wrap.call(req, &self.handler)
    }
}
