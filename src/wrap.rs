use crate::error::IntoResponseError;
use crate::handler::ErasedHandler;
use crate::http::{Request, Response};
use crate::{bounded, Error};

#[crate::async_trait]
pub trait Wrap<'req>: bounded::Send + bounded::Sync {
    type Error: IntoResponseError<'req>;

    async fn call(
        &self,
        req: &'req Request,
        next: impl Next<'req>,
    ) -> Result<Response, Self::Error>
    where
        'async_trait: 'req;
}

#[crate::async_trait]
pub trait Next<'req>: bounded::Send + bounded::Sync + 'req {
    async fn call(self, req: &'req Request) -> Result<Response, Error<'req>>;
}

#[non_exhaustive]
pub struct Call;

impl Call {
    pub fn new() -> Self {
        Self
    }
}

#[crate::async_trait]
impl<'req> Wrap<'req> for Call {
    type Error = Error<'req>;

    async fn call(
        &self,
        req: &'req Request,
        next: impl Next<'req>,
    ) -> Result<Response, Self::Error> {
        next.call(req).await
    }
}

pub struct And<I, O> {
    pub inner: I,
    pub outer: O,
}

#[crate::async_trait]
impl<'req, I, O> Wrap<'req> for And<I, O>
where
    I: Wrap<'req>,
    O: Wrap<'req>,
{
    type Error = O::Error;

    async fn call(&self, req: &'req Request, next: impl Next<'req>) -> Result<Response, Self::Error>
    where
        'async_trait: 'req,
    {
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

#[crate::async_trait]
impl<'req, I, O> Next<'req> for And<I, &'req O>
where
    O: Wrap<'req>,
    I: Next<'req>,
{
    async fn call(self, req: &'req Request) -> Result<Response, Error<'req>> {
        self.outer
            .call(req, self.inner)
            .await
            .map_err(IntoResponseError::into_response_error)
    }
}

pub struct DynNext<'bison>(&'bison dyn ErasedHandler);

impl<'bison> DynNext<'bison> {
    pub fn new(handler: &'bison dyn ErasedHandler) -> Self {
        Self(handler)
    }
}

#[crate::async_trait]
impl<'req> Next<'req> for DynNext<'req> {
    async fn call(self, req: &'req Request) -> Result<Response, Error> {
        self.0.call(req).await
    }
}
