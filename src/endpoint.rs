use crate::send::{BoxFuture, Boxed, SendBound};
use crate::wrap::Wrap;
use crate::{Error, HasContext, Request, Response, ResponseError};

use std::future::Future;
use std::marker::PhantomData;

pub trait Endpoint<C>: SendBound
where
    C: HasContext,
{
    /// An error that can occur during extraction.
    ///
    /// This is either a type implementing [`ResponseError`] or a boxed [`Error`].
    type Error: Into<Error>;

    fn serve(&self, context: C) -> BoxFuture<'_, Result<Response, Self::Error>>;

    fn wrap<W>(self, wrap: W) -> Wrapped<Self, C, W>
    where
        W: Wrap,
        Self: Sized,
    {
        Wrapped::new(self, wrap)
    }
}

impl<H, C, F, E> Endpoint<C> for H
where
    H: Fn(C) -> F + SendBound,
    C: HasContext,
    F: Future<Output = Result<Response, E>> + SendBound + 'static,
    E: ResponseError,
{
    type Error = E;

    fn serve(&self, context: C) -> BoxFuture<'_, Result<Response, Self::Error>> {
        self(context).boxed()
    }
}

impl<C, E> Endpoint<C> for Box<dyn Endpoint<C, Error = E>>
where
    C: HasContext + 'static,
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

impl<C, E> Endpoint<C> for Ref<'_, E>
where
    E: Endpoint<C>,
    C: HasContext,
{
    type Error = E::Error;

    fn serve(&self, context: C) -> BoxFuture<'_, Result<Response, Self::Error>> {
        self.endpoint.serve(context)
    }
}

/// An endpoint with context.
pub(crate) struct WithContext<E, C> {
    endpoint: E,
    _ctx: PhantomData<C>,
}

impl<E, C> WithContext<E, C>
where
    E: Endpoint<C>,
    C: HasContext,
{
    pub(crate) fn new(endpoint: E) -> Self {
        Self {
            endpoint,
            _ctx: PhantomData,
        }
    }
}

impl<E, C> Endpoint<Request> for WithContext<E, C>
where
    E: Endpoint<C> + SendBound,
    C: HasContext + SendBound,
{
    type Error = Error;

    fn serve(&self, req: Request) -> BoxFuture<'_, Result<Response, Error>> {
        Box::pin(async move {
            let ctx = C::construct(req).await.map_err(Error::new)?;
            let call = self.endpoint.serve(ctx);
            call.await.map_err(Into::into)
        })
    }
}

/// A handler wrapped with middleware.
pub struct Wrapped<E, C, W> {
    wrap: W,
    endpoint: WithContext<E, C>,
}

impl<E, C, W> Wrapped<E, C, W>
where
    E: Endpoint<C>,
    C: HasContext,
{
    pub(crate) fn new(endpoint: E, wrap: W) -> Self {
        Self {
            wrap,
            endpoint: WithContext::new(endpoint),
        }
    }
}

impl<E, C, W> Endpoint<Request> for Wrapped<E, C, W>
where
    W: Wrap + 'static,
    E: Endpoint<C>,
    C: HasContext,
{
    type Error = Error;

    fn serve(&self, req: Request) -> BoxFuture<'_, Result<Response, Error>> {
        Box::pin(async move {
            self.wrap
                .wrap(req, Ref::new(&self.endpoint))
                .await
                .map_err(Into::into)
        })
    }
}
