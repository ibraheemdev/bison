use super::NoArgument;
use crate::bounded::{Send, Sync};
use crate::http::{Body, Request, Response, ResponseBuilder, StatusCode};
use crate::ResponseError;

use std::fmt;

pub fn body<T>(req: &Request, param: NoArgument) -> Result<T, BodyError<T::Error>>
where
    T: FromBody,
{
    let body = req.body().take().ok_or(BodyError(BodyErrorKind::Taken))?;

    T::from_body(body).map_err(|e| BodyError(BodyErrorKind::FromBody(e)))
}

pub trait FromBody: Sized {
    type Error: fmt::Debug + fmt::Display + Send + Sync;

    fn from_body(body: Body) -> Result<Self, Self::Error>;
}

#[derive(Debug)]
pub struct BodyError<E>(BodyErrorKind<E>);

#[derive(Debug)]
enum BodyErrorKind<E> {
    Taken,
    FromBody(E),
}

impl<E> fmt::Display for BodyError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            BodyErrorKind::Taken => write!(
                f,
                "cannot have two request body extractors for a single handler",
            ),
            BodyErrorKind::FromBody(err) => {
                write!(f, "failed to extract body from request: {}", err)
            }
        }
    }
}

impl<E> ResponseError for BodyError<E>
where
    E: fmt::Debug + fmt::Display + Send + Sync,
{
    fn respond(self: Box<Self>) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}
