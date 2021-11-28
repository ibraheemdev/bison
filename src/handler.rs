use crate::error::IntoResponseError;
use crate::http::{Request, Response};
use crate::{bounded, wrap, Context, Error, Responder, WithContext, Wrap};

use std::future::Future;
use std::marker::PhantomData;

#[crate::async_trait_internal]
pub trait Handler<'req, C: WithContext<'req>>: bounded::Send + bounded::Sync {
    type Response: Responder;

    async fn call(&self, req: C::Context) -> Self::Response
    where
        'req: 'async_trait;
}

pub trait HandlerExt<C>: for<'req> Handler<'req, C> + 'static
where
    C: for<'req> WithContext<'req>,
{
    fn wrap<W>(self, wrap: W) -> Wrapped<C, W>
    where
        W: Wrap,
        Self: Sized,
    {
        Wrapped {
            wrap,
            handler: erase(self),
            _cx: PhantomData,
        }
    }
}

impl<H, C> HandlerExt<C> for H
where
    H: for<'req> Handler<'req, C> + 'static,
    C: for<'req> WithContext<'req>,
{
}

impl<'req, 'any> Handler<'req, &'any Request> for Box<ErasedHandler> {
    type Response = Response;

    fn call<'a, 'o>(&'a self, req: &'req Request) -> bounded::BoxFuture<'o, Response>
    where
        'a: 'o,
        'req: 'o,
    {
        (&**self).call(req)
    }
}

pub struct Wrapped<C, W> {
    wrap: W,
    handler: Box<ErasedHandler>,
    _cx: PhantomData<C>,
}

#[crate::async_trait_internal]
impl<'req, 'any, C, W> Handler<'req, &'any Request> for Wrapped<C, W>
where
    W: Wrap,
    C: WithContext<'req> + bounded::Send + bounded::Sync,
{
    type Response = Response;

    async fn call(&self, req: &'req Request) -> Self::Response {
        match self.wrap.call(req, wrap::DynNext(&*self.handler)).await {
            Ok(response) => response,
            Err(err) => {
                let mut err = Error::from(err.into_response_error());
                let mut response = err.respond();
                response.extensions_mut().insert(HandlerError(Some(err)));
                response
            }
        }
    }
}

#[crate::async_trait_internal]
impl<'req, F, O, R> Handler<'req, ()> for F
where
    F: Fn() -> O + bounded::Send + bounded::Sync + 'static,
    O: Future<Output = R> + bounded::Send + 'req,
    R: Responder,
{
    type Response = R;

    async fn call(&self, _: ()) -> Self::Response {
        self().await
    }
}

#[crate::async_trait_internal]
impl<'req, F, C, O, R> Handler<'req, (C,)> for F
where
    F: FnArgs<C>,
    F: Fn(<C as WithContext<'req>>::Context) -> O + bounded::Send + bounded::Sync + 'static,
    O: Future<Output = R> + bounded::Send + 'req,
    R: Responder,
    C: WithContext<'req>,
{
    type Response = R;

    async fn call(&self, req: C::Context) -> Self::Response
    where
        'req: 'async_trait,
    {
        self(req).await
    }
}

pub struct HandlerFn<F, H> {
    handler: H,
    f: F,
}

impl<H, F> HandlerFn<F, H> {
    pub fn new(handler: H, f: F) -> Self
    where
        F: for<'req> Fn(&'req H, &'req Request) -> bounded::BoxFuture<'req, Response>,
        H: bounded::Send + bounded::Sync,
    {
        Self { handler, f }
    }
}

#[crate::async_trait_internal]
impl<'req, F, H> Handler<'req, &'req Request> for HandlerFn<F, H>
where
    F: for<'r> Fn(&'r H, &'r Request) -> bounded::BoxFuture<'r, Response>
        + bounded::Send
        + bounded::Sync
        + 'static,
    H: bounded::Send + bounded::Sync,
{
    type Response = Response;

    async fn call(&self, req: &'req Request) -> Self::Response {
        (self.f)(&self.handler, req).await
    }
}

pub struct HandlerError(pub Option<Error>);

pub type ErasedHandler = dyn for<'req> Handler<'req, &'req Request, Response = Response>;

pub fn erase<H, C>(handler: H) -> Box<ErasedHandler>
where
    H: for<'req> Handler<'req, C> + 'static,
    C: for<'req> WithContext<'req>,
{
    Box::new(HandlerFn::new(handler, {
        move |handler, req| {
            Box::pin(async {
                match C::Context::extract(req).await {
                    Ok(context) => handler.call(context).await.respond(req),
                    Err(mut err) => {
                        let mut response = err.respond();
                        response.extensions_mut().insert(HandlerError(Some(err)));
                        response
                    }
                }
            })
        }
    }))
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
