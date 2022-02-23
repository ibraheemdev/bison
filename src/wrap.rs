use crate::bounded::{BoxFuture, Rc, Send};
use crate::{Handler, Request, Result};

/// Asynchronous HTTP middleware.
#[async_trait::async_trait]
pub trait Wrap: Send + Sync + 'static {
    /// Call the middleware with a request, and the next
    /// handler in the chain.nd a handler.
    async fn call(&self, req: Request, next: &impl Next) -> Result;

    /// Add another middleware to the chain.
    ///
    /// The returned middleware will pass `self` to
    /// the given middleware as the next parameter.
    fn wrap<W>(self, wrap: W) -> And<Self, W>
    where
        W: Wrap,
        Self: Sized,
    {
        And {
            inner: self,
            outer: wrap,
        }
    }
}

impl<W> Wrap for Rc<W>
where
    W: Wrap,
{
    fn call<'a, 'b, 'o>(&'a self, req: Request, next: &'b impl Next) -> BoxFuture<'o, Result>
    where
        'a: 'o,
        'b: 'o,
    {
        Wrap::call(&**self, req, next)
    }
}

/// The next middleware in the chain.
#[async_trait::async_trait]
pub trait Next: Send + Sync {
    /// Call this middleware with the given request.
    async fn call(&self, req: Request) -> Result;
}

impl<H> Next for H
where
    H: Handler,
{
    fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result>
    where
        'a: 'o,
    {
        Handler::call(self, req)
    }
}

/// A combination of two middlewares.
///
/// See [`Wrap::wrap`] for details.
pub struct And<I, O> {
    inner: I,
    outer: O,
}

#[async_trait::async_trait]
impl<I, O> Wrap for And<I, O>
where
    I: Wrap,
    O: Wrap,
{
    async fn call(&self, req: Request, next: &impl Next) -> Result {
        self.outer
            .call(
                req,
                &And {
                    inner: next,
                    outer: &self.inner,
                },
            )
            .await
    }
}

impl<I, O> Next for And<&I, &O>
where
    I: Next,
    O: Wrap,
{
    fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result>
    where
        'a: 'o,
    {
        self.outer.call(req, self.inner)
    }
}

/// Middleware that calls next with no extra processing.
///
/// This is useful in generic code as a base middleware type.
pub struct Call;

#[async_trait::async_trait]
impl Wrap for Call {
    async fn call(&self, req: Request, next: &impl Next) -> Result {
        next.call(req).await
    }
}

pub(crate) struct DynNext<'h>(pub &'h dyn Handler);

impl<'h> Next for DynNext<'h> {
    fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result>
    where
        'a: 'o,
    {
        Handler::call(self.0, req)
    }
}
