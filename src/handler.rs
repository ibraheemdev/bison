use crate::context::WithContext;
use crate::http::{Request, Response};
use crate::{bounded, Error};

use std::pin::Pin;

pub trait Handler<'req, C>: Clone + bounded::Send + bounded::Sync + 'static
where
    C: WithContext<'req>,
{
    fn call(&self, req: C::Context) -> Pin<Box<dyn bounded::Future<Output = Response> + 'req>>;
}

impl<'req, F, O, C> Handler<'req, C> for F
where
    // https://github.com/rust-lang/rust/issues/90875
    F: FnArgs<C>,
    F: Fn(C::Context) -> O + bounded::Send + bounded::Sync + Clone + 'static,
    O: bounded::Future<Output = Response> + 'req,
    C: WithContext<'req>,
{
    fn call(&self, req: C::Context) -> Pin<Box<dyn bounded::Future<Output = Response> + 'req>> {
        Box::pin(self(req))
    }
}

pub struct HandlerFn<F> {
    f: F,
}

impl<F> HandlerFn<F> {
    pub fn new(f: F) -> Self
    where
        F: for<'req> Fn(&'req Request) -> Pin<Box<dyn bounded::Future<Output = Response> + 'req>>,
    {
        Self { f }
    }
}

pub trait ErasedHandler: bounded::Send + bounded::Sync {
    fn call<'req>(
        &self,
        req: &'req Request,
    ) -> Pin<Box<dyn bounded::Future<Output = Result<Response, Error>> + 'req>>;
}

impl<F> ErasedHandler for HandlerFn<F>
where
    F: for<'req> Fn(&'req Request) -> Pin<Box<dyn bounded::Future<Output = Response> + 'req>>
        + bounded::Send
        + bounded::Sync,
{
    fn call<'req>(
        &self,
        req: &'req Request,
    ) -> Pin<Box<dyn bounded::Future<Output = Result<Response, Error>> + 'req>> {
        let fut = (self.f)(req);
        Box::pin(async move { Ok(fut.await) })
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
