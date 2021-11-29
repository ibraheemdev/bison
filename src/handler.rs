use crate::bounded::{BoxFuture, Send, Sync};
use crate::error::IntoResponseError;
use crate::http::{Request, Response};
use crate::{wrap, Context, Error, Responder, WithContext, Wrap};

use std::convert::Infallible;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{self, Poll};

pub trait Handler<'a, C>: Send + Sync
where
    C: WithContext<'a>,
{
    type Response: Responder;
    type Error: IntoResponseError;
    type Future: Future<Output = Result<Self::Response, Self::Error>> + Send + 'a;

    fn call(&'a self, cx: C::Context) -> Self::Future;

    fn wrap<W>(self, wrap: W) -> Wrapped<Self, C, W>
    where
        W: Wrap,
        Self: Sized,
    {
        Wrapped {
            wrap,
            handler: Extract::new(self),
            _cx: PhantomData,
        }
    }
}

impl<'a, 'any> Handler<'a, &'any Request> for Box<Erased> {
    type Response = Response;
    type Error = Error;
    type Future = BoxFuture<'a, Result<Response, Error>>;

    fn call(&'a self, req: &'a Request) -> Self::Future {
        (&**self).call(req)
    }
}

pub struct Wrapped<H, C, W> {
    wrap: W,
    handler: Extract<H, C>,
    _cx: PhantomData<C>,
}

impl<'a, 'any, H, C, W> Handler<'a, &'any Request> for Wrapped<H, C, W>
where
    W: Wrap,
    H: for<'b> Handler<'b, C>,
    C: for<'b> WithContext<'b> + Send + Sync,
{
    type Response = Response;
    type Error = Error;
    type Future = WrappedFuture<BoxFuture<'a, Result<Response, W::Error>>, W::Error>;

    fn call(&'a self, req: &'a Request) -> Self::Future {
        WrappedFuture {
            future: self.wrap.call(req, &self.handler),
            _e: PhantomData,
        }
    }
}

pin_project_lite::pin_project! {
    pub struct WrappedFuture<F, E> {
        #[pin]
        future: F,
        _e: PhantomData<E>
    }
}

impl<F, E> Future for WrappedFuture<F, E>
where
    F: Future<Output = Result<Response, E>> + Send,
    E: IntoResponseError,
{
    type Output = Result<Response, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.project()
            .future
            .poll(cx)
            .map_err(|err| err.into_response_error())
    }
}

pin_project_lite::pin_project! {
    pub struct InfallibleFut<F> {
        #[pin]
        future: F,
    }
}

impl<F: Future> Future for InfallibleFut<F> {
    type Output = Result<F::Output, Infallible>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.project().future.poll(cx).map(Ok)
    }
}

impl<'a, F, O, R> Handler<'a, ()> for F
where
    F: Fn() -> O + Send + Sync,
    O: Future<Output = R> + Send + 'a,
    R: Responder,
{
    type Response = R;
    type Future = InfallibleFut<O>;
    type Error = Infallible;

    fn call(&'a self, _: ()) -> Self::Future {
        InfallibleFut { future: self() }
    }
}

impl<'a, F, C, O, R> Handler<'a, (C,)> for F
where
    F: FnArgs<C>,
    F: Fn(<C as WithContext<'a>>::Context) -> O + Send + Sync,
    O: Future<Output = R> + Send + 'a,
    R: Responder,
    C: WithContext<'a>,
{
    type Response = R;
    type Future = InfallibleFut<O>;
    type Error = Infallible;

    fn call(&self, req: C::Context) -> Self::Future {
        InfallibleFut { future: self(req) }
    }
}

pub type Erased = dyn for<'a> Handler<
    'a,
    &'a Request,
    Response = Response,
    Error = Error,
    Future = BoxFuture<'a, Result<Response, Error>>,
>;

pub struct Boxed<H> {
    handler: H,
}

impl<H> Boxed<H> {
    pub fn new(handler: H) -> Boxed<H> {
        Boxed { handler }
    }
}

impl<'a, H> Handler<'a, &'a Request> for Boxed<H>
where
    for<'b> H: Handler<'b, &'b Request, Response = Response, Error = Error>,
{
    type Response = Response;
    type Error = Error;
    type Future = BoxFuture<'a, Result<Response, Error>>;

    fn call(&'a self, req: &'a Request) -> Self::Future {
        Box::pin(async move { self.handler.call(req).await })
    }
}

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
            let poll = match self.as_mut().project().state.project() {
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
                        responder
                            .respond(self.request)
                            .map_err(|e| e.into_response_error()),
                    ),
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err.into_response_error())),
                    Poll::Pending => Poll::Pending,
                },
            };

            break poll;
        }
    }
}

pub trait FnArgs<A> {
    fn call(&self, args: A);
}

impl<F, O, A> FnArgs<A> for F
where
    F: Fn(A) -> O,
{
    fn call(&self, args: A) {
        self(args);
    }
}
