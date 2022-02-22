use std::borrow::Cow;
use std::convert::Infallible;

use crate::http::header::ContentType;
use crate::http::{Body, Bytes, Response, StatusCode};
use crate::reject::{IntoRejection, NotFound, Rejection};

/// A type that can be converted into an HTTP response.
pub trait Respond {
    /// An error that can occur during the conversion.
    type Rejection: IntoRejection;

    /// Convert into an HTTP response.
    fn respond(self) -> Result<Response, Self::Rejection>;

    /// Returns a new responder that adds the provided status
    /// code to the response.
    fn with_status(self, status: StatusCode) -> (StatusCode, Self)
    where
        Self: Sized,
    {
        (status, self)
    }
}

impl Respond for () {
    type Rejection = Infallible;

    fn respond(self) -> Result<Response, Infallible> {
        Ok(Response::default())
    }
}

impl Respond for Response {
    type Rejection = Infallible;

    fn respond(self) -> Result<Response, Infallible> {
        Ok(self)
    }
}

impl<T> Respond for (StatusCode, T)
where
    T: Respond,
{
    type Rejection = T::Rejection;

    fn respond(self) -> Result<Response, T::Rejection> {
        self.1.respond().map(|mut response| {
            response.status = self.0;
            response
        })
    }
}

impl<T, E> Respond for Result<T, E>
where
    T: Respond,
    E: IntoRejection,
{
    type Rejection = Rejection;

    fn respond(self) -> Result<Response, Rejection> {
        self.map_err(Rejection::new)
            .and_then(|ok| ok.respond().map_err(Rejection::new))
    }
}

impl<T> Respond for Option<T>
where
    T: Respond,
{
    type Rejection = Rejection;

    fn respond(self) -> Result<Response, Rejection> {
        match self {
            Some(responder) => responder.respond().map_err(Rejection::new),
            None => Err(NotFound.into()),
        }
    }
}

macro_rules! respond {
    ($($ty:ty $(|> $into:ident)? => $content_type:expr),* $(,)?) => { $(
        impl Respond for $ty {
            type Rejection = Infallible;

            fn respond(self) -> Result<Response, Infallible> {
                Ok(Response::builder()
                    .header($content_type)
                    .body(Body::new(self $(.$into())?)))
            }
        })*
    }
}

respond! {
    Bytes => ContentType::OctetStream,
    Vec<u8> => ContentType::OctetStream,
    &'static [u8] => ContentType::OctetStream,
    Cow<'static, [u8]> |> into_owned => ContentType::OctetStream,
    String => ContentType::Text,
    &'static str => ContentType::Text,
    Cow<'static, str> |> into_owned => ContentType::Text,
}
