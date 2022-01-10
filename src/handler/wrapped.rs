use crate::handler::{Context, Extract, Handler};
use crate::http::{Request, Response};
use crate::reject::IntoRejection;
use crate::{Rejection, Wrap};

use std::marker::PhantomData;

/// A handler wrapped with some middleware.
pub struct Wrapped<H, C, W, WC> {
    wrap: W,
    handler: Extract<H, C>,
    _cx: PhantomData<(C, WC)>,
}

impl<H, C, W, WC> Wrapped<H, C, W, WC> {
    pub(crate) fn new(handler: H, wrap: W) -> Self {
        Wrapped {
            wrap,
            handler: Extract::new(handler),
            _cx: PhantomData,
        }
    }
}

#[crate::async_trait_internal]
impl<H, C, W, WC> Handler<Request> for Wrapped<H, C, W, WC>
where
    W: Wrap<WC>,
    H: Handler<C>,
    C: Context,
    WC: Context,
{
    type Response = Response;
    type Rejection = Rejection;

    async fn call(&self, req: Request) -> Result<Response, Rejection> {
        let cx = WC::extract(req).await?;

        self.wrap
            .call(cx, &self.handler)
            .await
            .map_err(|e| e.into_response_error())
    }
}
