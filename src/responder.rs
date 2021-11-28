use crate::error::{Error, IntoResponseError};
use crate::handler::HandlerError;
use crate::http::{Body, Request, Response};

pub trait Responder {
    fn respond(self, req: &Request) -> Response;
}

impl Responder for Response {
    fn respond(self, _: &Request) -> Response {
        self
    }
}

impl Responder for String {
    fn respond(self, _: &Request) -> Response {
        Response::new(Body::once(self))
    }
}

impl<T, E> Responder for Result<T, E>
where
    T: Responder,
    E: IntoResponseError,
{
    fn respond(self, req: &Request) -> Response {
        match self {
            Ok(ok) => ok.respond(req),
            Err(err) => {
                let mut err = Error::from(err.into_response_error());
                let mut response = err.respond();
                response.extensions_mut().insert(HandlerError(Some(err)));
                response
            }
        }
    }
}
