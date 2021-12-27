use crate::bounded::BoxFuture;
use crate::http::{Request, Response};
use crate::{AnyResponseError, Handler};

/// A completely type-erased `Handler`.
pub type Erased = dyn for<'a> Handler<
    'a,
    &'a Request,
    Response = Response,
    Error = AnyResponseError,
    Future = BoxFuture<'a, Result<Response, AnyResponseError>>,
>;

impl<'a, 'any> Handler<'a, &'any Request> for Box<Erased> {
    type Response = Response;
    type Error = AnyResponseError;
    type Future = BoxFuture<'a, Result<Response, AnyResponseError>>;

    fn call(&'a self, req: &'a Request) -> Self::Future {
        (&**self).call(req)
    }
}

/// A handler that returns a type-erased `BoxFuture`.
pub struct BoxReturn<H> {
    handler: H,
}

impl<H> BoxReturn<H> {
    pub(crate) fn new(handler: H) -> BoxReturn<H> {
        BoxReturn { handler }
    }
}

impl<'a, H> Handler<'a, &'a Request> for BoxReturn<H>
where
    for<'b> H: Handler<'b, &'b Request, Response = Response, Error = AnyResponseError>,
{
    type Response = Response;
    type Error = AnyResponseError;
    type Future = BoxFuture<'a, Result<Response, AnyResponseError>>;

    fn call(&'a self, req: &'a Request) -> Self::Future {
        Box::pin(async move { self.handler.call(req).await })
    }
}
