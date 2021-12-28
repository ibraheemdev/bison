use std::borrow::Cow;
use std::convert::Infallible;

use crate::error::{Error, IntoResponseError, NotFound};
use crate::http::{header, Body, Bytes, Response, ResponseBuilder, StatusCode};

/// A type that can be converted into an HTTP response.
pub trait Responder {
    /// An error that can occur during the conversion.
    type Error: IntoResponseError;

    /// Convert into an HTTP response.
    fn respond(self) -> Result<Response, Self::Error>;
}

impl Responder for () {
    type Error = Infallible;

    fn respond(self) -> Result<Response, Infallible> {
        Ok(Response::default())
    }
}

impl Responder for Response {
    type Error = Infallible;

    fn respond(self) -> Result<Response, Infallible> {
        Ok(self)
    }
}

impl<T> Responder for (StatusCode, T)
where
    T: Responder,
{
    type Error = T::Error;

    fn respond(self) -> Result<Response, T::Error> {
        self.1.respond().map(|mut response| {
            *response.status_mut() = self.0;
            response
        })
    }
}

impl<T, E> Responder for Result<T, E>
where
    T: Responder,
    E: IntoResponseError,
{
    type Error = Error;

    fn respond(self) -> Result<Response, Error> {
        self.map_err(Error::new)
            .and_then(|ok| ok.respond().map_err(Error::new))
    }
}

impl<T> Responder for Option<T>
where
    T: Responder,
{
    type Error = Error;

    fn respond(self) -> Result<Response, Error> {
        match self {
            Some(responder) => responder.respond().map_err(Error::new),
            None => Err(NotFound.into()),
        }
    }
}

macro_rules! with_content_type {
    ($($ty:ty $(|$into:ident)? => $content_type:literal),* $(,)?) => { $(
        impl Responder for $ty {
            type Error = Infallible;

            fn respond(self) -> Result<Response, Infallible> {
                Ok(ResponseBuilder::new()
                    .header(header::CONTENT_TYPE, $content_type)
                    .body(Body::once(self $(.$into())?))
                    .unwrap())
            }
        })*
    }
}

with_content_type! {
    Bytes => "application/octet-stream",
    Vec<u8> => "application/octet-stream",
    &'static [u8] => "application/octet-stream",
    Cow<'static, [u8]> | into_owned => "application/octet-stream",
    String => "text/plain",
    &'static str => "text/plain",
    Cow<'static, str> | into_owned => "text/plain",
}
