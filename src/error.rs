use crate::http::Response;

use std::convert::Infallible;
use std::fmt::{self, Debug, Display};

pub trait ResponseError: Debug + Display {
    fn respond(&mut self) -> Response;
}

// TODO
// impl ResponseError for Response {
//     fn respond(&mut self) -> Response {
//         std::mem::take(self)
//     }
// }

impl ResponseError for Infallible {
    fn respond(&mut self) -> Response {
        unreachable!()
    }
}

impl<'a> ResponseError for Box<dyn ResponseError + 'a> {
    fn respond(&mut self) -> Response {
        (&mut **self).respond()
    }
}

pub struct Error<'req> {
    inner: Box<dyn ResponseError + 'req>,
}

impl<'req> fmt::Debug for Error<'req> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl<'req> fmt::Display for Error<'req> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<'req> Error<'req> {
    pub fn new(err: impl ResponseError + 'req) -> Self {
        Self {
            inner: Box::new(err),
        }
    }

    pub fn respond(mut self) -> Response {
        self.inner.respond()
    }
}

impl<'req, E> From<E> for Error<'req>
where
    E: ResponseError + 'req,
{
    fn from(err: E) -> Self {
        Self {
            inner: Box::new(err),
        }
    }
}

pub trait IntoResponseError<'req> {
    fn into_response_error(self) -> Error<'req>;
}

impl<'req, E> IntoResponseError<'req> for E
where
    E: ResponseError + 'req,
{
    fn into_response_error(self) -> Error<'req> {
        self.into()
    }
}

impl<'req> IntoResponseError<'req> for Error<'req> {
    fn into_response_error(self) -> Error<'req> {
        self
    }
}
