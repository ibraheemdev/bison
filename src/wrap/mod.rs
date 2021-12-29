//! Asynchronous middleware.

mod wrap_fn;

#[doc(hidden)]
pub use wrap_fn::__internal_wrap_fn;

use crate::bounded::{BoxFuture, Rc, Send, Sync};
use crate::handler::Erased;
use crate::http::{Request, Response};
use crate::reject::IntoRejection;
use crate::{Rejection, Responder};

/// Middleware that wraps around the rest of the chain.
#[crate::async_trait_internal]
pub trait Wrap: Send + Sync + 'static {
    /// An error that can occur when calling this middleware.
    type Rejection: IntoRejection;

    /// Call the middleware with a request, and the next
    /// handler in the chain.
    async fn call(&self, req: &Request, next: &impl Next) -> Result<Response, Self::Rejection>;

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
    async fn call(&self, req: &Request) -> Result<Response, Rejection>;
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

    async fn call(&self, req: &Request, next: &impl Next) -> Result<Response, Self::Rejection> {
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

    async fn call(&self, req: &Request, next: &impl Next) -> Result<Response, Self::Rejection> {
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
impl<'r, I, O> Next for And<I, &'r O>
where
    O: Wrap,
    I: Next,
{
    async fn call(&self, req: &Request) -> Result<Response, Rejection> {
        self.outer
            .call(req, &self.inner)
            .await
            .map_err(|e| e.into_response_error())
            .and_then(|r| r.respond().map_err(|e| e.into_response_error()))
    }
}

pub(crate) struct DynNext<'bison>(pub &'bison Erased);

impl<'bison> DynNext<'bison> {
    pub fn new(handler: &'bison Erased) -> Self {
        Self(handler)
    }
}

#[crate::async_trait_internal]
impl<'bison> Next for DynNext<'bison> {
    async fn call(&self, req: &Request) -> Result<Response, Rejection> {
        self.0.call(req).await
    }
}

impl<W: Wrap> Wrap for Rc<W> {
    type Rejection = W::Rejection;

    fn call<'a, 'b, 'c, 'o>(
        &'a self,
        req: &'b Request,
        next: &'c impl Next,
    ) -> BoxFuture<'o, Result<Response, Self::Rejection>>
    where
        'a: 'o,
        'b: 'o,
        'c: 'o,
    {
        W::call(self, req, next)
    }
}

impl<I: Next> Next for &I {
    fn call<'a, 'b, 'o>(&'a self, req: &'b Request) -> BoxFuture<'o, Result<Response, Rejection>>
    where
        'a: 'o,
        'b: 'o,
    {
        I::call(self, req)
    }
}
