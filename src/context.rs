use crate::http::Request;
use crate::{bounded, Error};

use std::pin::Pin;

pub trait Context<'r>: bounded::Send + bounded::Sync + Sized {
    fn extract(
        req: &'r Request,
    ) -> Pin<Box<dyn bounded::Future<Output = Result<Self, Error>> + 'r>>;
}

pub trait WithContext<'r> {
    type Context: Context<'r>;
}

impl<'r> Context<'r> for &'r Request {
    fn extract(
        req: &'r Request,
    ) -> Pin<Box<dyn bounded::Future<Output = Result<Self, Error>> + 'r>> {
        Box::pin(async move { Ok(req) })
    }
}

impl<'r> Context<'r> for () {
    fn extract(_: &'r Request) -> Pin<Box<dyn bounded::Future<Output = Result<Self, Error>> + 'r>> {
        Box::pin(async move { Ok(()) })
    }
}

impl<'r> WithContext<'r> for () {
    type Context = ();
}

impl<'r, T: WithContext<'r>> WithContext<'r> for (T,) {
    type Context = T::Context;
}

impl<'any, 'r> WithContext<'r> for &'any Request {
    type Context = &'r Request;
}
