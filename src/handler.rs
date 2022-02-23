use crate::bounded::{BoxFuture, Send};
use crate::{Request, Result, Wrap};

use std::future::Future;

/// An asynchronous HTTP handler.
#[async_trait::async_trait]
pub trait Handler<'req>: Send + Sync {
    /// Call this handler with the given request.
    async fn call(&self, req: &'req mut Request) -> Result;

    /// Wrap this handler in some middleware.
    fn wrap<W>(self, wrap: W) -> Wrapped<Self, W>
    where
        Self: Sized,
        W: Wrap<'req>,
    {
        Wrapped {
            handler: self,
            wrap,
        }
    }
}

#[async_trait::async_trait]
impl<'req, F, Fut> Handler<'req> for F
where
    F: Fn(&'req mut Request) -> Fut + Send + Sync,
    Fut: Future<Output = Result> + Send + Sync,
{
    async fn call(&self, req: &'req mut Request) -> Result {
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

#[async_trait::async_trait]
impl<'req, H, W> Handler<'req> for Wrapped<H, W>
where
    H: Handler<'req>,
    W: Wrap<'req>,
{
    async fn call(&self, req: &'req mut Request) -> Result {
        self.wrap.wrap(req, HandlerRef(&self.handler)).await
    }
}

/// A reference to a handler.
pub struct HandlerRef<'h, H>(pub &'h H);

impl<'h, 'req, H> Handler<'req> for HandlerRef<'h, H>
where
    H: Handler<'req>,
{
    fn call<'a, 'o>(&self, req: &'req mut Request) -> BoxFuture<'o, Result>
    where
        'a: 'o,
        'h: 'o,
        'req: 'o,
    {
        self.0.call(req)
    }
}
