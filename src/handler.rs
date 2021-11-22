use crate::bounded;
use crate::context::WithContext;
use crate::error::{Error, IntoResponseError};
use crate::http::{Request, Response};

use std::future::Future;

pub trait Handler<'r, C>: Clone + bounded::Send + bounded::Sync + 'static
where
    C: WithContext<'r>,
{
    type Error: IntoResponseError<'r>;

    fn call(&self, req: C::Context) -> bounded::BoxFuture<'r, Result<Response, Self::Error>>;
}

impl<'r, F, O, E> Handler<'r, ()> for F
where
    F: Fn() -> O + bounded::Send + bounded::Sync + Clone + 'static,
    O: Future<Output = Result<Response, E>> + bounded::Send + bounded::Sync + 'r,
    E: IntoResponseError<'r>,
{
    type Error = E;

    fn call(&self, _: ()) -> bounded::BoxFuture<'r, Result<Response, Self::Error>> {
        Box::pin(self())
    }
}

impl<'r, F, C, O, E> Handler<'r, (C,)> for F
where
    // https://github.com/rust-lang/rust/issues/90875
    F: FnArgs<C>,
    F: Fn(C::Context) -> O + bounded::Send + bounded::Sync + Clone + 'static,
    O: Future<Output = Result<Response, E>> + bounded::Send + bounded::Sync + 'r,
    E: IntoResponseError<'r>,
    C: WithContext<'r>,
{
    type Error = E;

    fn call(&self, req: C::Context) -> bounded::BoxFuture<'r, Result<Response, Self::Error>> {
        Box::pin(self(req))
    }
}

pub struct HandlerFn<F> {
    f: F,
}

impl<F> HandlerFn<F> {
    pub fn new(f: F) -> Self
    where
        F: for<'r> Fn(&'r Request) -> bounded::BoxFuture<'r, Result<Response, Error>>,
    {
        Self { f }
    }
}

pub trait ErasedHandler: bounded::Send + bounded::Sync {
    fn call<'r>(&'r self, req: &'r Request) -> bounded::BoxFuture<'r, Result<Response, Error>>;
}

impl<F> ErasedHandler for HandlerFn<F>
where
    F: for<'r> Fn(&'r Request) -> bounded::BoxFuture<'r, Result<Response, Error>>
        + bounded::Send
        + bounded::Sync,
{
    fn call<'r>(&'r self, req: &'r Request) -> bounded::BoxFuture<'r, Result<Response, Error>> {
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
