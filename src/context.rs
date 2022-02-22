use crate::http::Request;
use crate::reject::Reject;

use std::convert::Infallible;

pub trait Context<'req>: Sized {
    type Reject: Reject;

    fn extract(req: &'req Request) -> Result<Self, Self::Reject>;
}

impl<'req> Context<'req> for &'req Request {
    type Reject = Infallible;

    fn extract(req: &'req Request) -> Result<Self, Self::Reject> {
        Ok(req)
    }
}

pub trait ContextMut<'req>: Sized {
    type Reject: Reject;

    fn extract_mut(req: &'req mut Request) -> Result<Self, Self::Reject>;
}

impl<'req> ContextMut<'req> for &'req mut Request {
    type Reject = Infallible;

    fn extract_mut(req: &'req mut Request) -> Result<Self, Self::Reject> {
        Ok(req)
    }
}
