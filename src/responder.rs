use std::convert::Infallible;

use crate::error::{Error, IntoResponseError};
use crate::http::{Body, Request, Response};

pub trait Responder {
    type Error: IntoResponseError;

    fn respond(self, req: &Request) -> Result<Response, Self::Error>;
}

impl Responder for Response {
    type Error = Infallible;

    fn respond(self, _: &Request) -> Result<Response, Infallible> {
        Ok(self)
    }
}

impl Responder for String {
    type Error = Infallible;

    fn respond(self, _: &Request) -> Result<Response, Infallible> {
        Ok(Response::new(Body::once(self)))
    }
}

impl<T, E> Responder for Result<T, E>
where
    T: Responder,
    E: IntoResponseError,
{
    type Error = Error;

    fn respond(self, req: &Request) -> Result<Response, Error> {
        self.map_err(|err| err.into_response_error())
            .and_then(|ok| ok.respond(req).map_err(|err| err.into_response_error()))
    }
}
