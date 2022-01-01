//! Asynchronous middleware.

mod wrap_fn;

#[doc(hidden)]
pub use wrap_fn::__internal_wrap_fn;

use crate::bounded::{BoxFuture, Rc, Send, Sync};
use crate::handler::Handler;
use crate::http::{Request, Response};
use crate::reject::IntoRejection;
use crate::{Rejection, Respond};

/// Middleware that wraps around the rest of the chain.
#[crate::async_trait_internal]
pub trait Wrap: Send + Sync + 'static {
    /// An error that can occur when calling this middleware.
    type Rejection: IntoRejection;

    /// Call the middleware with a request, and the next
    /// handler in the chain.
    async fn call(&self, req: Request, next: &impl Next) -> Result<Response, Self::Rejection>;

    /// Add another middleware to the chain.
    ///
    /// The returned middleware will pass `self` to
    /// the given middleware as the next parameter.
    fn and<W>(self, wrap: W) -> And<Self, W>
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

/// The next handler in the middleware chain.
#[crate::async_trait_internal]
pub trait Next: Send + Sync {
    /// Call the handler with the request.
    async fn call(&self, req: Request) -> Result<Response, Rejection>;
}

/// Middleware that calls next with no extra processing.
///
/// This is useful in generic code as a base middleware type.
#[non_exhaustive]
pub struct Call;

impl Call {
    /// Create a new instance of this type.
    pub fn new() -> Self {
        Self
    }
}

#[crate::async_trait_internal]
impl Wrap for Call {
    type Rejection = Rejection;

    async fn call(&self, req: Request, next: &impl Next) -> Result<Response, Self::Rejection> {
        next.call(req).await
    }
}

/// A combination of two middlewares.
///
/// See [`Wrap::and`] for details.
pub struct And<I, O> {
    inner: I,
    outer: O,
}

#[crate::async_trait_internal]
impl<I, O> Wrap for And<I, O>
where
    I: Wrap,
    O: Wrap,
{
    type Rejection = O::Rejection;

    async fn call(&self, req: Request, next: &impl Next) -> Result<Response, Self::Rejection> {
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

#[crate::async_trait_internal]
impl<I, O> Next for And<&I, &O>
where
    O: Wrap,
    I: Next,
{
    async fn call(&self, req: Request) -> Result<Response, Rejection> {
        self.outer
            .call(req, self.inner)
            .await
            .map_err(|e| e.into_response_error())
            .and_then(|r| r.respond().map_err(|e| e.into_response_error()))
    }
}

#[crate::async_trait_internal]
impl<W: Wrap> Wrap for Rc<W> {
    type Rejection = W::Rejection;

    fn call<'a, 'b, 'o>(
        &'a self,
        req: Request,
        next: &'b impl Next,
    ) -> BoxFuture<'o, Result<Response, Self::Rejection>>
    where
        'a: 'o,
        'b: 'o,
    {
        W::call(self, req, next)
    }
}

impl<H> Next for H
where
    H: Handler<Request, Response = Response, Rejection = Rejection>,
{
    fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result<Response, Rejection>>
    where
        'a: 'o,
    {
        Handler::call(self, req)
    }
}
