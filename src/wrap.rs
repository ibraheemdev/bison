use crate::bounded::{BoxFuture, Rc, Send, Sync};
use crate::error::IntoResponseError;
use crate::handler::Erased;
use crate::http::{Request, Response};
use crate::{Error, Responder};

use std::future::Future;
use std::marker::PhantomData;

#[crate::async_trait_internal]
pub trait Wrap: Send + Sync + 'static {
    type Error: IntoResponseError;

    async fn call<'a>(&self, req: &Request, next: impl Next + 'a) -> Result<Response, Self::Error>;
}

impl<W: Wrap> Wrap for Rc<W> {
    type Error = W::Error;

    fn call<'a, 'b, 'c, 'o>(
        &'b self,
        req: &'c Request,
        next: impl Next + 'a,
    ) -> BoxFuture<'o, Result<Response, Self::Error>>
    where
        'a: 'o,
        'b: 'o,
        'c: 'o,
    {
        W::call(self, req, next)
    }
}

#[crate::async_trait_internal]
pub trait Next: Send + Sync {
    async fn call(&self, req: &Request) -> Result<Response, Error>;
}

impl<I: Next> Next for &I {
    fn call<'a, 'b, 'o>(&'a self, req: &'b Request) -> BoxFuture<'o, Result<Response, Error>>
    where
        'a: 'o,
        'b: 'o,
    {
        I::call(self, req)
    }
}

#[non_exhaustive]
pub struct Call;

impl Call {
    pub fn new() -> Self {
        Self
    }
}

#[crate::async_trait_internal]
impl Wrap for Call {
    type Error = Error;

    async fn call<'a>(&self, req: &Request, next: impl Next + 'a) -> Result<Response, Self::Error> {
        next.call(req).await
    }
}

pub struct And<I, O> {
    pub inner: I,
    pub outer: O,
}

#[crate::async_trait_internal]
impl<I, O> Wrap for And<I, O>
where
    I: Wrap,
    O: Wrap,
{
    type Error = O::Error;

    async fn call<'a>(&self, req: &Request, next: impl Next + 'a) -> Result<Response, Self::Error> {
        self.outer
            .call(
                req,
                And {
                    inner: next,
                    outer: &self.inner,
                },
            )
            .await
    }
}

#[crate::async_trait_internal]
impl<'r, I, O> Next for And<I, &'r O>
where
    O: Wrap,
    I: Next,
{
    async fn call(&self, req: &Request) -> Result<Response, Error> {
        self.outer
            .call(req, &self.inner)
            .await
            .map_err(|e| e.into_response_error())
            .and_then(|r| r.respond(req).map_err(|e| e.into_response_error()))
    }
}

pub struct DynNext<'bison>(pub &'bison Erased);

impl<'bison> DynNext<'bison> {
    pub fn new(handler: &'bison Erased) -> Self {
        Self(handler)
    }
}

#[crate::async_trait_internal]
impl<'bison> Next for DynNext<'bison> {
    async fn call(&self, req: &Request) -> Result<Response, Error> {
        self.0.call(req).await
    }
}

pub trait WrapFn<'a, E>: Send + Sync + 'static {
    type F: Future<Output = Result<Response, E>> + Send + 'a;

    fn call(&self, req: &'a Request, next: &'a dyn Next) -> Self::F;
}

impl<'a, O, E, F> WrapFn<'a, E> for F
where
    F: Fn(&'a Request, &'a dyn Next) -> O + Send + Sync + 'static,
    O: Future<Output = Result<Response, E>> + Send + 'a,
    E: IntoResponseError,
{
    type F = O;

    fn call(&self, req: &'a Request, next: &'a dyn Next) -> Self::F {
        self(req, next)
    }
}

pub fn wrap_fn<F, E>(f: F) -> impl Wrap
where
    for<'a> F: WrapFn<'a, E>,
    E: IntoResponseError + Send + Sync + 'static,
{
    struct WrapFnImpl<F, E>(F, PhantomData<E>);

    #[crate::async_trait_internal]
    impl<F, E> Wrap for WrapFnImpl<F, E>
    where
        F: for<'a> WrapFn<'a, E>,
        E: IntoResponseError + Send + Sync + 'static,
    {
        type Error = E;

        async fn call<'b>(
            &self,
            req: &Request,
            next: impl Next + 'b,
        ) -> Result<Response, Self::Error> {
            self.0.call(req, &next).await
        }
    }

    WrapFnImpl(f, PhantomData::<E>)
}
