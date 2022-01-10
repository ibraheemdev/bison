//! This module is full of hacks around compiler limitations
//! with HRTBs :(

use crate::bounded::{Send, Sync};
use crate::handler::Handler;
use crate::{Context, Rejection, Respond, Response};

use std::future::Future;

#[crate::async_trait_internal]
impl<F, O, R> Handler<()> for F
where
    F: Fn() -> O + Send + Sync + 'static,
    O: Future<Output = R> + Send,
    R: Respond,
{
    type Response = Response;
    type Rejection = Rejection;

    async fn call(&self, _: ()) -> Result<Response, Rejection> {
        self().await.respond().map_err(Rejection::new)
    }
}

#[crate::async_trait_internal]
impl<'req, F, C, O, R> Handler<(C,)> for F
where
    F: Fn(C) -> O + Send + Sync + 'static,
    O: Future<Output = R> + Send + 'req,
    C: Context<'req>,
    R: Respond,
{
    type Response = Response;
    type Rejection = Rejection;

    async fn call(&self, (cx,): (C,)) -> Result<Response, Rejection>
    where
        C: 'async_trait,
    {
        self(cx).await.respond().map_err(Rejection::new)
    }
}
