use crate::error::IntoResponseError;
use crate::handler::ErasedHandler;
use crate::http::{Request, Response};
use crate::{bounded, Error};

#[crate::async_trait]
pub trait Wrap<'r>: bounded::Send + bounded::Sync {
    type Error: IntoResponseError<'r>;

    async fn call(
        &self,
        req: &'r Request,
        next: impl Next<'r>,
    ) -> Result<Response, Self::Error>
    where
        'async_trait: 'r;
}

#[crate::async_trait]
pub trait Next<'r>: bounded::Send + bounded::Sync + 'r {
    async fn call(self, req: &'r Request) -> Result<Response, Error<'r>>;
}

#[non_exhaustive]
pub struct Call;

impl Call {
    pub fn new() -> Self {
        Self
    }
}

#[crate::async_trait]
impl<'r> Wrap<'r> for Call {
    type Error = Error<'r>;

    async fn call(
        &self,
        req: &'r Request,
        next: impl Next<'r>,
    ) -> Result<Response, Self::Error> {
        next.call(req).await
    }
}

pub struct And<I, O> {
    pub inner: I,
    pub outer: O,
}

#[crate::async_trait]
impl<'r, I, O> Wrap<'r> for And<I, O>
where
    I: Wrap<'r>,
    O: Wrap<'r>,
{
    type Error = O::Error;

    async fn call(&self, req: &'r Request, next: impl Next<'r>) -> Result<Response, Self::Error>
    where
        'async_trait: 'r,
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
impl<'r, I, O> Next<'r> for And<I, &'r O>
where
    O: Wrap<'r>,
    I: Next<'r>,
{
    async fn call(self, req: &'r Request) -> Result<Response, Error<'r>> {
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
impl<'r> Next<'r> for DynNext<'r> {
    async fn call(self, req: &'r Request) -> Result<Response, Error> {
        self.0.call(req).await
    }
}
