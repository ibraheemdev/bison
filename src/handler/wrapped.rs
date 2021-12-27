use super::{Extract, Handler};
use crate::bounded::{BoxFuture, Send, Sync};
use crate::error::IntoResponseError;
use crate::http::{Request, Response};
use crate::{AnyResponseError, WithContext, Wrap};

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{self, Poll};

/// A handler wrapped with some middleware.
pub struct Wrapped<H, C, W> {
    wrap: W,
    handler: Extract<H, C>,
    _cx: PhantomData<C>,
}

impl<H, C, W> Wrapped<H, C, W> {
    pub(crate) fn new(handler: H, wrap: W) -> Self {
        Wrapped {
            wrap,
            handler: Extract::new(handler),
            _cx: PhantomData,
        }
    }
}

impl<'a, 'any, H, C, W> Handler<'a, &'any Request> for Wrapped<H, C, W>
where
    W: Wrap,
    H: for<'b> Handler<'b, C>,
    C: for<'b> WithContext<'b> + Send + Sync,
{
    type Response = Response;
    type Error = AnyResponseError;
    type Future = IntoResponseErrorFut<BoxFuture<'a, Result<Response, W::Error>>, W::Error>;

    fn call(&'a self, req: &'a Request) -> Self::Future {
        IntoResponseErrorFut {
            future: self.wrap.call(req, &self.handler),
            _e: PhantomData,
        }
    }
}

pin_project_lite::pin_project! {
    pub struct IntoResponseErrorFut<F, E> {
        #[pin]
        future: F,
        _e: PhantomData<E>
    }
}

impl<F, E> Future for IntoResponseErrorFut<F, E>
where
    F: Future<Output = Result<Response, E>> + Send,
    E: IntoResponseError,
{
    type Output = Result<Response, AnyResponseError>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.project()
            .future
            .poll(cx)
            .map_err(|err| err.into_response_error())
    }
}
