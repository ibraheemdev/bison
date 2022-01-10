use crate::bounded::BoxFuture;
use crate::handler::{Context, Extract, Handler};
use crate::http::{Request, Response};
use crate::Rejection;

pub fn erase<H, C>(handler: H) -> Box<Erased>
where
    H: Handler<C>,
    C: Context,
{
    Box::new(Extract::new(handler))
}

/// A completely type-erased `Handler`.
pub type Erased = dyn Handler<Request, Response = Response, Rejection = Rejection>;

impl Handler<Request> for Box<Erased> {
    type Response = Response;
    type Rejection = Rejection;

    fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result<Response, Rejection>>
    where
        'a: 'o,
    {
        (&**self).call(req)
    }
}
