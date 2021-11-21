use crate::bounded;
use crate::http::Request;

use std::pin::Pin;

pub trait Context<'req>: bounded::Send + bounded::Sync {
    fn extract(req: &'req Request) -> Pin<Box<dyn bounded::Future<Output = Self> + 'req>>;
}

pub trait WithContext<'req> {
    type Context: Context<'req>;
}

impl<'req> Context<'req> for &'req Request {
    fn extract(req: &'req Request) -> Pin<Box<dyn bounded::Future<Output = Self> + 'req>> {
        Box::pin(async move { req })
    }
}

impl<'any, 'req> WithContext<'req> for &'any Request {
    type Context = &'req Request;
}
