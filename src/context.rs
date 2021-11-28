use crate::http::Request;
use crate::{bounded, Error};

pub trait Context<'req>: bounded::Send + bounded::Sync + Sized {
    fn extract(req: &'req Request) -> bounded::BoxFuture<'req, Result<Self, Error>>;
}

pub trait WithContext<'req> {
    type Context: Context<'req> + 'req;
}

impl<'req> Context<'req> for &'req Request {
    fn extract(req: &'req Request) -> bounded::BoxFuture<'req, Result<Self, Error>> {
        Box::pin(async move { Ok(req) })
    }
}

impl<'req> Context<'req> for () {
    fn extract(_: &'req Request) -> bounded::BoxFuture<'req, Result<Self, Error>> {
        Box::pin(async move { Ok(()) })
    }
}

impl<'req> WithContext<'req> for () {
    type Context = ();
}

impl<'req, T: WithContext<'req>> WithContext<'req> for (T,) {
    type Context = T::Context;
}

impl<'any, 'req> WithContext<'req> for &'any Request {
    type Context = &'req Request;
}
