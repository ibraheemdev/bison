use crate::bounded::BoxFuture;
use crate::handler::{Context, Handler};
use crate::http::{Request, Response};
use crate::{Rejection, Respond};

use std::marker::{PhantomData, PhantomPinned};
use std::mem;

pub unsafe trait Static {}
unsafe impl<T> Static for T where T: Unpin + 'static {}
unsafe impl Static for PhantomPinned {}

/// # Safety
///
/// - 'req must act as an HRTB.
/// - The handler must be 'static, other than context PhantomData
pub unsafe fn erase<'req, H, C>(handler: H, _guard: &'req ()) -> Erased
where
    H: Handler<C> + 'req + Static,
    C: Context<'req>,
{
    let handler: Box<
        dyn Handler<&'req Request, Response = Response, Rejection = Rejection> + 'req,
    > = Box::new(Extract::new(handler));

    // SAFETY:
    //
    // - Extract stores only `H`, which is 'static, and `PhantomData<C>`
    // - 'req acts an HRTB, and so the handler cannot take advantage of
    // a 'static request
    Erased(unsafe { mem::transmute(handler) })
}

/// A type-erased `Handler`.
pub struct Erased(
    Box<dyn for<'req> Handler<&'req Request, Response = Response, Rejection = Rejection>>,
);

impl<'req> Handler<&'req Request> for Erased {
    type Response = Response;
    type Rejection = Rejection;

    fn call<'a, 'o>(&'a self, req: &'req Request) -> BoxFuture<'o, Result<Response, Rejection>>
    where
        'a: 'o,
        'req: 'o,
    {
        self.0.call(req)
    }
}

/// A that extracts context for an inner handler.
pub struct Extract<H, C>(H, PhantomData<C>);

impl<H, C> Extract<H, C> {
    pub fn new(handler: H) -> Self {
        Self(handler, PhantomData)
    }
}

#[crate::async_trait_internal]
impl<'req, H, C> Handler<&'req Request> for Extract<H, C>
where
    H: Handler<C>,
    C: Context<'req>,
{
    type Response = Response;
    type Rejection = Rejection;

    async fn call(&self, req: &'req Request) -> Result<Response, Rejection> {
        let cx = C::extract(&req).await?;
        self.0
            .call(cx)
            .await
            .map_err(Rejection::new)
            .and_then(|x| x.respond().map_err(Rejection::new))
    }
}
