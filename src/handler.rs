use crate::http::{Request, Response};
use crate::{bounded, Error, Responder, WithContext};

use std::future::Future;

pub trait Handler<'req, C>: Clone + bounded::Send + bounded::Sync + 'static
where
    C: WithContext<'req>,
{
    type Response: Responder<'req>;

    fn call(&self, req: C::Context) -> bounded::BoxFuture<'req, Self::Response>;
}

impl<'req, F, O, R> Handler<'req, ()> for F
where
    F: Fn() -> O + bounded::Send + bounded::Sync + Clone + 'static,
    O: Future<Output = R> + bounded::Send + bounded::Sync + 'req,
    R: Responder<'req>,
{
    type Response = R;

    fn call(&self, _: ()) -> bounded::BoxFuture<'req, Self::Response> {
        Box::pin(self())
    }
}

impl<'req, F, C, O, R> Handler<'req, (C,)> for F
where
    // https://github.com/rust-lang/rust/issues/90875
    F: FnArgs<C>,
    F: Fn(C::Context) -> O + bounded::Send + bounded::Sync + Clone + 'static,
    O: Future<Output = R> + bounded::Send + bounded::Sync + 'req,
    R: Responder<'req>,
    C: WithContext<'req>,
{
    type Response = R;

    fn call(&self, req: C::Context) -> bounded::BoxFuture<'req, Self::Response> {
        Box::pin(self(req))
    }
}

pub struct HandlerFn<F> {
    f: F,
}

impl<F> HandlerFn<F> {
    pub fn new(f: F) -> Self
    where
        F: for<'req> Fn(&'req Request) -> bounded::BoxFuture<'req, Result<Response, Error>>,
    {
        Self { f }
    }
}

pub trait ErasedHandler: bounded::Send + bounded::Sync {
    fn call<'req>(
        &'req self,
        req: &'req Request,
    ) -> bounded::BoxFuture<'req, Result<Response, Error>>;
}

impl<F> ErasedHandler for HandlerFn<F>
where
    F: for<'req> Fn(&'req Request) -> bounded::BoxFuture<'req, Result<Response, Error>>
        + bounded::Send
        + bounded::Sync,
{
    fn call<'req>(
        &'req self,
        req: &'req Request,
    ) -> bounded::BoxFuture<'req, Result<Response, Error>> {
        let fut = (self.f)(req);
        Box::pin(async move { fut.await })
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
