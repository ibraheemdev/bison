use crate::bison::State;
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
pub trait Wrap<S>: SendBound
where
    S: State,
{
    /// This is either a type implementing [`ResponseError`] or the boxed [`Error`]
    type Error: Into<Error>;

    fn wrap<'a>(
        &'a self,
        req: Request<S>,
        next: impl Next<S> + 'a,
    ) -> BoxFuture<'a, Result<Response, Self::Error>>;

    fn and<W>(self, other: W) -> And<Self, W>
    where
        W: Wrap<S>,
        Self: Sized,
    {
        And {
            inner: self,
            outer: other,
        }
    }
}

impl<S, W> Wrap<S> for &W
where
    S: State,
    W: Wrap<S>,
{
    type Error = W::Error;

    fn wrap<'a>(
        &'a self,
        req: Request<S>,
        next: impl Next<S> + 'a,
    ) -> BoxFuture<'a, Result<Response, Self::Error>> {
        W::wrap(self, req, next)
    }
}

pub trait Next<S>: Endpoint<Request<S>, S, Error = Error>
where
    S: State,
{
}

impl<S, E> Next<S> for E
where
    E: Endpoint<Request<S>, S, Error = Error>,
    S: State,
{
}

#[derive(Clone)]
pub struct And<I, O> {
    inner: I,
    outer: O,
}

impl<S, I, O> Wrap<S> for And<I, O>
where
    S: State,
    I: Wrap<S>,
    O: Wrap<S>,
{
    type Error = O::Error;

    fn wrap<'a>(
        &'a self,
        req: Request<S>,
        next: impl Next<S> + 'a,
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

impl<S, I, O> Endpoint<Request<S>, S> for And<I, O>
where
    S: State,
    O: Wrap<S>,
    I: Next<S>,
{
    type Error = Error;

    fn serve(&self, req: Request<S>) -> BoxFuture<'_, Result<Response, Error>> {
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

impl<S> Wrap<S> for Call
where
    S: State,
{
    type Error = Error;

    fn wrap<'a>(
        &'a self,
        req: Request<S>,
        next: impl Next<S> + 'a,
    ) -> BoxFuture<'a, Result<Response, Error>> {
        // this boxing is unfortunate because N::Future is already going to be boxed
        // however I'm not sure it's worth moving the type N onto Wrap<N> for, because
        // TAIT is coming soon
        Box::pin(async move { next.serve(req).await })
    }
}
