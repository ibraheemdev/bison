use crate::error::IntoResponseError;
use crate::http::{Request, Response};
use crate::{wrap, Context, Error, Handler, Responder, WithContext};

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{self, Poll};

/// Extracts the context from the request
/// needed to call a given handler.
pub struct Extract<H, C> {
    handler: H,
    _cx: PhantomData<C>,
}

impl<H, C> Extract<H, C> {
    pub fn new(handler: H) -> Extract<H, C> {
        Extract {
            handler,
            _cx: PhantomData,
        }
    }
}

#[crate::async_trait_internal]
impl<H, C> wrap::Next for Extract<H, C>
where
    H: for<'r> Handler<'r, C>,
    C: for<'r> WithContext<'r>,
{
    async fn call(&self, req: &Request) -> Result<Response, Error> {
        Handler::call(self, req).await
    }
}

impl<'a, H, C> Handler<'a, &'a Request> for Extract<H, C>
where
    H: for<'r> Handler<'r, C> + 'a,
    C: for<'r> WithContext<'r> + 'a,
{
    type Response = Response;
    type Error = Error;
    type Future = ExtractFut<'a, H, C>;

    fn call(&'a self, req: &'a Request) -> Self::Future {
        ExtractFut {
            state: ExtractFutState::Context {
                fut: C::Context::extract(req),
            },
            handler: &self.handler,
            request: req,
        }
    }
}

pin_project_lite::pin_project! {
    pub struct ExtractFut<'a, H, C>
    where
        H: Handler<'a, C>,
        C: WithContext<'a>
    {
        #[pin]
        state: ExtractFutState<H::Future, <C::Context as Context<'a>>::Future>,
        handler: &'a H,
        request: &'a Request
    }
}

pin_project_lite::pin_project! {
    #[project = ExtractProj]
    enum ExtractFutState<H, C> {
        Context {
            #[pin]
            fut: C,
        },
        Handler {
            #[pin]
            fut: H,
        },
    }
}

impl<'a, H, C> Future for ExtractFut<'a, H, C>
where
    H: Handler<'a, C>,
    C: WithContext<'a>,
{
    type Output = Result<Response, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        loop {
            break match self.as_mut().project().state.project() {
                ExtractProj::Context { fut } => match fut.poll(cx) {
                    Poll::Ready(Ok(cx)) => {
                        let fut = self.handler.call(cx);
                        self.as_mut()
                            .project()
                            .state
                            .set(ExtractFutState::Handler { fut });
                        continue;
                    }
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                    Poll::Pending => Poll::Pending,
                },
                ExtractProj::Handler { fut } => match fut.poll(cx) {
                    Poll::Ready(Ok(responder)) => Poll::Ready(
                        // TODO: move responding to separate handler
                        responder.respond().map_err(|e| e.into_response_error()),
                    ),
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err.into_response_error())),
                    Poll::Pending => Poll::Pending,
                },
            };
        }
    }
}
