use crate::handler::{Context, Extract, Handler};
use crate::http::{Request, Response};
use crate::reject::IntoRejection;
use crate::{Rejection, Wrap};

use std::marker::{PhantomData, PhantomPinned};

use super::erased::Static;

/// A handler wrapped with some middleware.
pub struct Wrapped<H, C, W, WC> {
    wrap: W,
    handler: Extract<H, C>,
    _cx: PhantomData<(C, WC)>,
    _x: PhantomPinned,
}

unsafe impl<H, C, W, WC> Static for Wrapped<H, C, W, WC>
where
    W: 'static,
    Extract<H, C>: 'static,
{
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
impl<'req, H, C, W, WC> Handler<&'req Request> for Wrapped<H, C, W, WC>
where
    W: Wrap<'req, WC>,
    H: Handler<C>,
    C: Context<'req>,
    WC: Context<'req>,
{
    type Response = Response;
    type Rejection = Rejection;

    async fn call(&self, req: &'req Request) -> Result<Response, Rejection> {
        let cx = WC::extract(&req).await?;

        self.wrap
            .call(cx, &self.handler)
            .await
            .map_err(|e| e.into_response_error())
    }
}
