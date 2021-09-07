use crate::bison::State;
use crate::http::IntoResponse;
use crate::send::{BoxFuture, Boxed, SendBound};
use crate::wrap::Wrap;
use crate::{Error, HasContext, Request, Response};

use std::future::Future;
use std::marker::PhantomData;

pub trait Endpoint<C, S>: SendBound
where
    S: State,
    C: HasContext<S>,
{
    /// An error that can occur during extraction.
    ///
    /// This is either a type implementing [`ResponseError`] or a boxed [`Error`].
    type Error: Into<Error>;

    fn serve(&self, context: C) -> BoxFuture<'_, Result<Response, Self::Error>>;

    fn wrap<W>(self, wrap: W) -> Wrapped<Self, C, W, S>
    where
        W: Wrap<S>,
        Self: Sized,
    {
        Wrapped::new(self, wrap)
    }
}

impl<H, S, C, F, R> Endpoint<C, S> for H
where
    S: State,
    H: Fn(C) -> F + SendBound,
    C: HasContext<S>,
    F: Future<Output = R> + SendBound + 'static,
    R: IntoResponse,
{
    type Error = Error;

    fn serve(&self, context: C) -> BoxFuture<'_, Result<Response, Self::Error>> {
        let fut = self(context);
        async move { fut.await.into_response() }.boxed()
    }
}

impl<S, C, E> Endpoint<C, S> for Box<dyn Endpoint<C, S, Error = E>>
where
    S: State,
    C: HasContext<S> + 'static,
    E: Into<Error> + 'static,
{
    type Error = E;

    fn serve(&self, context: C) -> BoxFuture<'_, Result<Response, Self::Error>> {
        (&**self).serve(context)
    }
}
/// A reference to an endpoint.
pub struct Ref<'a, E> {
    endpoint: &'a E,
}

impl<'a, E> Ref<'a, E> {
    pub fn new(endpoint: &'a E) -> Self {
        Self { endpoint }
    }
}

impl<S, C, E> Endpoint<C, S> for Ref<'_, E>
where
    S: State,
    E: Endpoint<C, S>,
    C: HasContext<S>,
{
    type Error = E::Error;

    fn serve(&self, context: C) -> BoxFuture<'_, Result<Response, Self::Error>> {
        self.endpoint.serve(context)
    }
}

/// An endpoint with context.
pub(crate) struct WithContext<E, C, S> {
    endpoint: E,
    _ctx: PhantomData<(C, S)>,
}

impl<S, E, C> WithContext<E, C, S>
where
    S: State,
    E: Endpoint<C, S>,
    C: HasContext<S>,
{
    pub(crate) fn new(endpoint: E) -> Self {
        Self {
            endpoint,
            _ctx: PhantomData,
        }
    }
}

impl<S, E, C> Endpoint<Request<S>, S> for WithContext<E, C, S>
where
    S: State,
    E: Endpoint<C, S> + SendBound,
    C: HasContext<S> + SendBound,
{
    type Error = Error;

    fn serve(&self, req: Request<S>) -> BoxFuture<'_, Result<Response, Error>> {
        Box::pin(async move {
            let ctx = C::extract(req).await.map_err(Into::into)?;
            let call = self.endpoint.serve(ctx);
            call.await.map_err(Into::into)
        })
    }
}

/// A handler wrapped with middleware.
pub struct Wrapped<E, C, W, S> {
    wrap: W,
    endpoint: WithContext<E, C, S>,
}

impl<S, E, C, W> Wrapped<E, C, W, S>
where
    S: State,
    E: Endpoint<C, S>,
    C: HasContext<S>,
{
    pub(crate) fn new(endpoint: E, wrap: W) -> Self {
        Self {
            wrap,
            endpoint: WithContext::new(endpoint),
        }
    }
}

impl<S, E, C, W> Endpoint<Request<S>, S> for Wrapped<E, C, W, S>
where
    S: State,
    W: Wrap<S> + 'static,
    E: Endpoint<C, S>,
    C: HasContext<S>,
{
    type Error = Error;

    fn serve(&self, req: Request<S>) -> BoxFuture<'_, Result<Response, Error>> {
        Box::pin(async move {
            self.wrap
                .wrap(req, Ref::new(&self.endpoint))
                .await
                .map_err(Into::into)
        })
    }
}
