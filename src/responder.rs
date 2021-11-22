use crate::error::IntoResponseError;
use crate::http::{Body, Request, Response};

pub trait Responder<'req> {
    fn respond(self, req: &'req Request) -> Response;
}

impl<'req> Responder<'req> for Response {
    fn respond(self, _: &'req Request) -> Response {
        self
    }
}

impl<'req> Responder<'req> for String {
    fn respond(self, _: &'req Request) -> Response {
        Response::new(Body::once(self))
    }
}

impl<'req, T, E> Responder<'req> for Result<T, E>
where
    T: Responder<'req>,
    E: IntoResponseError<'req>,
{
    fn respond(self, req: &'req Request) -> Response {
        match self {
            Ok(ok) => ok.respond(req),
            Err(err) => err.into_response_error().respond(),
        }
    }
}
