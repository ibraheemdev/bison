use crate::context::WithContext;
use crate::error::IntoResponseError;
use crate::http::{Request, Response};
use crate::{bounded, Error};

use std::pin::Pin;

pub trait Handler<'r, C>: Clone + bounded::Send + bounded::Sync + 'static
where
    C: WithContext<'r>,
{
    type Error: IntoResponseError<'r>;

    fn call(
        &self,
        req: C::Context,
    ) -> Pin<Box<dyn bounded::Future<Output = Result<Response, Self::Error>> + 'r>>;
}

impl<'r, F, O, E> Handler<'r, ()> for F
where
    F: Fn() -> O + bounded::Send + bounded::Sync + Clone + 'static,
    O: bounded::Future<Output = Result<Response, E>> + 'r,
    E: IntoResponseError<'r>,
{
    type Error = E;

    fn call(
        &self,
        _: (),
    ) -> Pin<Box<dyn bounded::Future<Output = Result<Response, Self::Error>> + 'r>> {
        Box::pin(self())
    }
}

impl<'r, F, C, O, E> Handler<'r, (C,)> for F
where
    // https://github.com/rust-lang/rust/issues/90875
    F: FnArgs<C>,
    F: Fn(C::Context) -> O + bounded::Send + bounded::Sync + Clone + 'static,
    O: bounded::Future<Output = Result<Response, E>> + 'r,
    E: IntoResponseError<'r>,
    C: WithContext<'r>,
{
    type Error = E;

    fn call(
        &self,
        req: C::Context,
    ) -> Pin<Box<dyn bounded::Future<Output = Result<Response, Self::Error>> + 'r>> {
        Box::pin(self(req))
    }
}

pub struct HandlerFn<F> {
    f: F,
}

impl<F> HandlerFn<F> {
    pub fn new(f: F) -> Self
    where
        F: for<'r> Fn(
            &'r Request,
        )
            -> Pin<Box<dyn bounded::Future<Output = Result<Response, Error>> + 'r>>,
    {
        Self { f }
    }
}

pub trait ErasedHandler: bounded::Send + bounded::Sync {
    fn call<'r>(
        &'r self,
        req: &'r Request,
    ) -> Pin<Box<dyn bounded::Future<Output = Result<Response, Error>> + 'r>>;
}

impl<F> ErasedHandler for HandlerFn<F>
where
    F: for<'r> Fn(
            &'r Request,
        ) -> Pin<Box<dyn bounded::Future<Output = Result<Response, Error>> + 'r>>
        + bounded::Send
        + bounded::Sync,
{
    fn call<'r>(
        &'r self,
        req: &'r Request,
    ) -> Pin<Box<dyn bounded::Future<Output = Result<Response, Error>> + 'r>> {
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
