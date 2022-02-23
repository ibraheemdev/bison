use crate::bounded::{BoxFuture, Rc, Send};
use crate::{Handler, Request, Result};

/// Asynchronous HTTP middleware.
#[async_trait::async_trait]
pub trait Wrap<'r>: Send + Sync + 'static {
    /// Call the middleware with a request, and the next
    /// handler in the chain.nd a handler.
    async fn call(&self, req: &'r mut Request, next: &impl Next<'r>) -> Result;

    /// Add another middleware to the chain.
    ///
    /// The returned middleware will pass `self` to
    /// the given middleware as the next parameter.
    fn wrap<W>(self, wrap: W) -> And<Self, W>
    where
        W: Wrap<'r>,
        Self: Sized,
    {
        And {
            inner: self,
            outer: wrap,
        }
    }
}

impl<'r, W> Wrap<'r> for Rc<W>
where
    W: Wrap<'r>,
{
    fn call<'a, 'b, 'o>(
        &'a self,
        req: &'r mut Request,
        next: &'b impl Next<'r>,
    ) -> BoxFuture<'o, Result>
    where
        'r: 'o,
        'a: 'o,
        'b: 'o,
    {
        Wrap::call(&**self, req, next)
    }
}

/// The next middleware in the chain.
#[async_trait::async_trait]
pub trait Next<'r>: Send + Sync {
    /// Call this middleware with the given request.
    async fn call(&self, req: &'r mut Request) -> Result;
}

impl<'r, H> Next<'r> for H
where
    H: Handler<'r>,
{
    fn call<'a, 'o>(&'a self, req: &'r mut Request) -> BoxFuture<'o, Result>
    where
        'r: 'o,
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
impl<'r, I, O> Wrap<'r> for And<I, O>
where
    I: Wrap<'r>,
    O: Wrap<'r>,
{
    async fn call(&self, req: &'r mut Request, next: &impl Next<'r>) -> Result {
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

impl<'r, I, O> Next<'r> for And<&I, &O>
where
    I: Next<'r>,
    O: Wrap<'r>,
{
    fn call<'a, 'o>(&'a self, req: &'r mut Request) -> BoxFuture<'o, Result>
    where
        'a: 'o,
        'r: 'o,
    {
        self.outer.call(req, self.inner)
    }
}

/// Middleware that calls next with no extra processing.
///
/// This is useful in generic code as a base middleware type.
pub struct Call;

#[async_trait::async_trait]
impl<'r> Wrap<'r> for Call {
    async fn call(&self, req: &'r mut Request, next: &impl Next<'r>) -> Result {
        next.call(req).await
    }
}

pub(crate) struct DynNext<'h>(pub &'h dyn for<'r> Handler<'r>);

impl<'h, 'r> Next<'r> for DynNext<'h> {
    fn call<'a, 'o>(&'a self, req: &'r mut Request) -> BoxFuture<'o, Result>
    where
        'a: 'o,
        'r: 'o,
    {
        Handler::call(self.0, req)
    }
}
