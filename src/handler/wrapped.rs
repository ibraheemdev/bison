use crate::handler::{Context, Extract, Handler};
use crate::http::{Request, Response};
use crate::reject::IntoRejection;
use crate::{Rejection, Wrap};

use std::marker::PhantomData;

/// A handler wrapped with some middleware.
pub struct Wrapped<H, C, W> {
    wrap: W,
    handler: Extract<H, C>,
    _cx: PhantomData<C>,
}

impl<H, C, W> Wrapped<H, C, W> {
    pub(crate) fn new(handler: H, wrap: W) -> Self {
        Wrapped {
            wrap,
            handler: Extract::new(handler),
            _cx: PhantomData,
        }
    }
}

#[crate::async_trait_internal]
impl<H, C, W> Handler<Request> for Wrapped<H, C, W>
where
    W: Wrap,
    H: Handler<C>,
    C: Context,
{
    type Response = Response;
    type Rejection = Rejection;

    async fn call(&self, req: Request) -> Result<Response, Rejection> {
        self.wrap
            .call(req, &self.handler)
            .await
            .map_err(|e| e.into_response_error())
    }
}
