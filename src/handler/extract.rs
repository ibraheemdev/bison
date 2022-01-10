use crate::handler::{Context, Handler};
use crate::http::{Request, Response};
use crate::{Rejection, Respond};

use std::marker::PhantomData;

pub struct Extract<H, C>(H, PhantomData<C>);

impl<H, C> Extract<H, C> {
    pub fn new(handler: H) -> Self {
        Self(handler, PhantomData)
    }
}

#[crate::async_trait_internal]
impl<H, C> Handler<Request> for Extract<H, C>
where
    H: Handler<C>,
    C: Context,
{
    type Response = Response;
    type Rejection = Rejection;

    async fn call(&self, req: Request) -> Result<Response, Rejection> {
        let cx = C::extract(req).await?;
        self.0
            .call(cx)
            .await
            .map_err(Rejection::new)
            .and_then(|x| x.respond().map_err(Rejection::new))
    }
}
