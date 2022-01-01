//! Asynchronous middleware.

mod wrap_fn;

use std::marker::PhantomData;

#[doc(hidden)]
pub use wrap_fn::__internal_wrap_fn;

use crate::bounded::{BoxFuture, Rc, Send, Sync};
use crate::handler::Handler;
use crate::http::{Request, Response};
use crate::reject::IntoRejection;
use crate::{Context, Rejection, Respond};

/// Middleware that wraps around the rest of the chain.
#[crate::async_trait_internal]
pub trait Wrap<C = Request>: Send + Sync + 'static {
    /// An error that can occur when calling this middleware.
    type Rejection: IntoRejection;

    /// Call the middleware with a request, and the next
    /// handler in the chain.
    async fn call(&self, cx: C, next: &impl Next) -> Result<Response, Self::Rejection>;

    /// Add another middleware to the chain.
    ///
    /// The returned middleware will pass `self` to
    /// the given middleware as the next parameter.
    fn wrap<W, O>(self, wrap: W) -> And<Self, W, C, O>
    where
        W: Wrap<O>,
        Self: Sized,
    {
        And {
            inner: self,
            outer: wrap,
            _cx: PhantomData,
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
impl Wrap<Request> for Call {
    type Rejection = Rejection;

    async fn call(&self, req: Request, next: &impl Next) -> Result<Response, Self::Rejection> {
        next.call(req).await
    }
}

/// A combination of two middlewares.
///
/// See [`Wrap::and`] for details.
pub struct And<I, O, IC, OC> {
    inner: I,
    outer: O,
    _cx: PhantomData<(IC, OC)>,
}

#[crate::async_trait_internal]
impl<I, O, IC, OC> Wrap<Request> for And<I, O, IC, OC>
where
    I: Wrap<IC>,
    O: Wrap<OC>,
    IC: Context,
    OC: Context,
{
    type Rejection = Rejection;

    async fn call(&self, req: Request, next: &impl Next) -> Result<Response, Self::Rejection> {
        let cx = OC::extract(req).await?;
        self.outer
            .call(
                cx,
                &And {
                    inner: next,
                    outer: &self.inner,
                    _cx: PhantomData::<(IC, OC)>,
                },
            )
            .await
            .map_err(Rejection::new)
    }
}

#[crate::async_trait_internal]
impl<I, O, IC, OC> Next for And<&I, &O, IC, OC>
where
    I: Next,
    O: Wrap<IC>,
    IC: Context,
    OC: Context,
{
    async fn call(&self, req: Request) -> Result<Response, Rejection> {
        let cx = IC::extract(req).await?;

        self.outer
            .call(cx, self.inner)
            .await
            .map_err(|e| e.into_response_error())
            .and_then(|r| r.respond().map_err(|e| e.into_response_error()))
    }
}

#[crate::async_trait_internal]
impl<W, C> Wrap<C> for Rc<W>
where
    W: Wrap<C>,
{
    type Rejection = W::Rejection;

    fn call<'a, 'b, 'o>(
        &'a self,
        cx: C,
        next: &'b impl Next,
    ) -> BoxFuture<'o, Result<Response, Self::Rejection>>
    where
        'a: 'o,
        'b: 'o,
    {
        W::call(self, cx, next)
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
