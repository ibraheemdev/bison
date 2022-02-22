use crate::context::ContextMut;
use crate::http::{Request, Response};
use crate::reject::{IntoRejection, Rejection};
use crate::respond::Respond;

use std::marker::PhantomData;

/// Middleware that wraps around the rest of the chain.
pub trait Wrap<'req, Context = &'req mut Request> {
    /// Error that can occur when calling this middleware.
    type Rejection: IntoRejection;

    /// Call the middleware with a request, and the next
    /// handler in the chain.
    fn call(&self, context: Context, next: impl Next<'req>) -> Result<Response, Self::Rejection>;

    /// Add another middleware to the chain.
    ///
    /// The returned middleware will pass `self` to
    /// the given middleware as the next parameter.
    fn and<W, O>(self, wrap: W) -> And<Self, W, Context, O>
    where
        W: Wrap<'req, O>,
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
pub trait Next<'req> {
    /// Call the handler with the request.
    fn call(&self, req: &'req mut Request) -> Result<Response, Rejection>;
}

impl<'req, N> Next<'req> for &N
where
    N: Next<'req>,
{
    fn call(&self, req: &'req mut Request) -> Result<Response, Rejection> {
        N::call(self, req)
    }
}

/// Middleware that calls next with no extra processing.
///
/// This is useful in generic code as a base middleware type.
pub struct Call;

impl<'req> Wrap<'req> for Call {
    type Rejection = Rejection;

    fn call(
        &self,
        req: &'req mut Request,
        next: impl Next<'req>,
    ) -> Result<Response, Self::Rejection> {
        next.call(req)
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

impl<'req, Inner, Outer, InnerCx, OuterCx> Wrap<'req> for And<Inner, Outer, InnerCx, OuterCx>
where
    Inner: Wrap<'req, InnerCx>,
    Outer: Wrap<'req, OuterCx>,
    InnerCx: ContextMut<'req>,
    OuterCx: ContextMut<'req>,
{
    type Rejection = Rejection;

    fn call(
        &self,
        req: &'req mut Request,
        next: impl Next<'req>,
    ) -> Result<Response, Self::Rejection> {
        let cx = OuterCx::extract_mut(req)?;
        let next = And {
            inner: next,
            outer: &self.inner,
            _cx: PhantomData::<(InnerCx, OuterCx)>,
        };

        self.outer.call(cx, next).map_err(Rejection::new)
    }
}

impl<'req, Inner, Outer, InnerCx, OuterCx> Next<'req> for And<Inner, &Outer, InnerCx, OuterCx>
where
    Inner: Next<'req>,
    Outer: Wrap<'req, InnerCx>,
    InnerCx: ContextMut<'req>,
    OuterCx: ContextMut<'req>,
{
    fn call(&self, req: &'req mut Request) -> Result<Response, Rejection> {
        let cx = InnerCx::extract_mut(req)?;

        self.outer
            .call(cx, &self.inner)
            .map_err(|e| e.into_response_error())
            .and_then(|r| r.respond().map_err(|e| e.into_response_error()))
    }
}
