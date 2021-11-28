use crate::http::Request;
use crate::{bounded, Error};

use std::future::{ready, Future, Ready};

pub trait Context<'req>: bounded::Send + bounded::Sync + Sized {
    type Future: Future<Output = Result<Self, Error>> + bounded::Send + 'req;

    fn extract(req: &'req Request) -> Self::Future;
}

pub trait WithContext<'req>: bounded::Send + bounded::Sync {
    type Context: Context<'req> + 'req;
}

impl<'req> Context<'req> for &'req Request {
    type Future = Ready<Result<Self, Error>>;

    fn extract(req: &'req Request) -> Self::Future {
        ready(Ok(req))
    }
}

impl<'req> Context<'req> for () {
    type Future = Ready<Result<Self, Error>>;

    fn extract(_: &'req Request) -> Self::Future {
        ready(Ok(()))
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
