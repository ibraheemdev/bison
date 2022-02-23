use crate::bounded::{BoxFuture, Send};
use crate::wrap::Next;
use crate::{Request, Result, Wrap};

use std::future::Future;
use std::marker::PhantomData;

/// An asynchronous HTTP handler.
#[async_trait::async_trait]
pub trait Handler<S = ()>: Send + Sync {
    /// Call this handler with the given request.
    async fn call(&self, req: Request) -> Result;

    /// Wrap this handler in some middleware.
    fn wrap<W>(self, wrap: W) -> Wrapped<Self, W, S>
    where
        Self: Sized,
        W: Wrap,
    {
        Wrapped {
            wrap,
            handler: NextHandler(self, PhantomData),
            _state: PhantomData,
        }
    }

    /// Returns a type-erased handler.
    fn boxed(self) -> BoxHandler
    where
        Self: Sized + 'static,
        S: Send + Sync + 'static,
    {
        struct Impl<H, S>(H, PhantomData<S>);

        impl<H, S> Handler for Impl<H, S>
        where
            H: Handler<S>,
            S: Send + Sync,
        {
            fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result>
            where
                'a: 'o,
            {
                self.0.call(req)
            }
        }

        Box::new(Impl(self, PhantomData))
    }
}

/// A type-erased [`Handler`].
pub type BoxHandler = Box<dyn Handler>;

impl<S> Handler<S> for BoxHandler {
    fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result>
    where
        'a: 'o,
    {
        Handler::call(&**self, req)
    }
}

pub trait State: Send + Sync {
    fn extract(req: &Request) -> Self;
}

#[async_trait::async_trait]
impl<F, Fut, S> Handler<S> for F
where
    F: Fn(S, Request) -> Fut + Send + Sync,
    Fut: Future<Output = Result> + Send + Sync,
    S: State,
{
    async fn call(&self, req: Request) -> Result {
        let state = S::extract(&req);
        self(state, req).await
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
pub struct Wrapped<H, W, S> {
    handler: NextHandler<H, S>,
    wrap: W,
    _state: PhantomData<S>,
}

impl<H, W, S> Handler<S> for Wrapped<H, W, S>
where
    H: Handler<S>,
    W: Wrap,
    S: Send + Sync + 'static,
{
    fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result>
    where
        'a: 'o,
    {
        self.wrap.call(req, &self.handler)
    }
}

struct NextHandler<H, S>(H, PhantomData<S>);

impl<H, S> Next for NextHandler<H, S>
where
    H: Handler<S>,
    S: Send + Sync + 'static,
{
    fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result>
    where
        'a: 'o,
    {
        Handler::call(&self.0, req)
    }
}
