use crate::error::IntoResponseError;
use crate::http::{Request, Response};
use crate::{bounded, wrap, Context, Error, Responder, WithContext, Wrap};

use std::future::Future;
use std::marker::PhantomData;

#[crate::async_trait_internal]
pub trait Handler<'req, C>: bounded::Send + bounded::Sync
where
    C: WithContext<'req>,
{
    type Response: Responder;

    async fn call(&self, req: C::Context) -> Self::Response
    where
        'req: 'async_trait;
}

pub trait HandlerExt<C>: for<'req> Handler<'req, C> + 'static
where
    C: for<'req> WithContext<'req>,
{
    fn wrap<W>(self, wrap: W) -> Wrapped<Self, C, W>
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

impl<'req, 'any> Handler<'req, &'any Request> for Box<Erased> {
    type Response = Response;

    fn call<'a, 'o>(&'a self, req: &'req Request) -> bounded::BoxFuture<'o, Response>
    where
        'a: 'o,
        'req: 'o,
    {
        (&**self).call(req)
    }
}

pub struct Wrapped<H, C, W> {
    wrap: W,
    handler: Erase<H, C>,
    _cx: PhantomData<C>,
}

#[crate::async_trait_internal]
impl<'req, 'any, H, C, W> Handler<'req, &'any Request> for Wrapped<H, C, W>
where
    W: Wrap,
    H: for<'r> Handler<'r, C> + 'static,
    C: for<'r> WithContext<'r> + bounded::Send + bounded::Sync,
{
    type Response = Response;

    async fn call(&self, req: &'req Request) -> Self::Response {
        match self.wrap.call(req, &self.handler).await {
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

pub struct HandlerError(pub Option<Error>);

pub type Erased = dyn for<'req> Handler<'req, &'req Request, Response = Response>;

pub struct Erase<H, C> {
    handler: H,
    _cx: PhantomData<C>,
}

pub fn erase<H, C>(handler: H) -> Erase<H, C>
where
    H: for<'r> Handler<'r, C>,
    C: for<'r> WithContext<'r>,
{
    Erase {
        handler,
        _cx: PhantomData,
    }
}

#[crate::async_trait_internal]
impl<H, C> wrap::Next for Erase<H, C>
where
    H: for<'r> Handler<'r, C>,
    C: for<'r> WithContext<'r>,
{
    async fn call(&self, req: &Request) -> Result<Response, Error> {
        let mut response = Handler::call(self, req).await;

        if let Some(error) = response.extensions_mut().get_mut::<HandlerError>() {
            return Err(error.0.take().unwrap());
        }

        Ok(response)
    }
}

#[crate::async_trait_internal]
impl<'req, H, C> Handler<'req, &'req Request> for Erase<H, C>
where
    H: for<'r> Handler<'r, C>,
    C: for<'r> WithContext<'r>,
{
    type Response = Response;

    async fn call(&self, req: &'req Request) -> Self::Response {
        match C::Context::extract(req).await {
            Ok(context) => self.handler.call(context).await.respond(req),
            Err(mut err) => {
                let mut response = err.respond();
                response.extensions_mut().insert(HandlerError(Some(err)));
                response
            }
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
