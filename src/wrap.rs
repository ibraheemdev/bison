use crate::endpoint::{Endpoint, Ref};
use crate::send::{BoxFuture, SendBound};
use crate::{Error, Request, Response};

/// An HTTP middleware.
///
/// ```rust
/// struct LoggerMiddleware;
///
/// impl Middleware for LoggerMiddleware {
///     type Error = ();
///
///     fn call<'a>(
///         &'a self,
///         request: Request,
///         next: impl Next + 'a
///     ) -> BoxFuture<'a, Result<Response, ()>> {
///         async move {
///             let before = Instant::now();
///             let response = next.call(request).await;
///             let response_time = Instant::now() - before;
///             log::info!("Requested '{}', response time: {}", request.path(), response_time);
///         }.boxed()
///     }
/// }
/// ```
pub trait Wrap: SendBound {
    /// This is either a type implementing [`ResponseError`] or the boxed [`Error`]
    type Error: Into<Error>;

    fn wrap<'a>(
        &'a self,
        req: Request,
        next: impl Next + 'a,
    ) -> BoxFuture<'a, Result<Response, Self::Error>>;

    fn and<W>(self, other: W) -> And<Self, W>
    where
        W: Wrap,
        Self: Sized,
    {
        And {
            inner: self,
            outer: other,
        }
    }
}

impl<W> Wrap for &W
where
    W: Wrap + SendBound,
{
    type Error = W::Error;

    fn wrap<'a>(
        &'a self,
        req: Request,
        next: impl Next + 'a,
    ) -> BoxFuture<'a, Result<Response, Self::Error>> {
        W::wrap(self, req, next)
    }
}

pub trait Next: Endpoint<Request, Error = Error> {}

impl<E> Next for E where E: Endpoint<Request, Error = Error> {}

#[derive(Clone)]
pub struct And<I, O> {
    inner: I,
    outer: O,
}

impl<I, O> Wrap for And<I, O>
where
    I: Wrap,
    O: Wrap,
{
    type Error = O::Error;

    fn wrap<'a>(
        &'a self,
        req: Request,
        next: impl Next + 'a,
    ) -> BoxFuture<'a, Result<Response, Self::Error>> {
        self.outer.wrap(
            req,
            And {
                inner: next,
                outer: &self.inner,
            },
        )
    }
}

impl<I, O> Endpoint<Request> for And<I, O>
where
    O: Wrap,
    I: Next,
{
    type Error = Error;

    fn serve(&self, req: Request) -> BoxFuture<'_, Result<Response, Error>> {
        Box::pin(async move {
            self.outer
                .wrap(req, Ref::new(&self.inner))
                .await
                .map_err(Into::into)
        })
    }
}

/// A middleware that simply calls `next`. This type is useful in generic code where a type
/// implement [`Middleware`] is expected.
#[derive(Clone)]
pub struct Call {
    _priv: (),
}

impl Call {
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl Wrap for Call {
    type Error = Error;

    fn wrap<'a>(
        &'a self,
        req: Request,
        next: impl Next + 'a,
    ) -> BoxFuture<'a, Result<Response, Error>> {
        // this boxing is unfortunate because N::Future is already going to be boxed
        // however I'm not sure it's worth moving the type N onto Wrap<N> for, because
        // TAIT is coming soon
        Box::pin(async move { next.serve(req).await })
    }
}
