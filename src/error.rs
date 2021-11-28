use crate::http::Response;

use std::convert::Infallible;
use std::fmt::{self, Debug, Display};

pub trait ResponseError: Debug + Display + Send + Sync {
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

pub struct Error {
    inner: Box<dyn ResponseError>,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Error {
    pub fn new(err: impl ResponseError + 'static) -> Self {
        Self {
            inner: Box::new(err),
        }
    }

    pub fn respond(&mut self) -> Response {
        self.inner.respond()
    }
}

impl<E> From<E> for Error
where
    E: ResponseError + 'static,
{
    fn from(err: E) -> Self {
        Self {
            inner: Box::new(err),
        }
    }
}

pub trait IntoResponseError {
    fn into_response_error(self) -> Error;
}

impl<E> IntoResponseError for E
where
    E: ResponseError + 'static,
{
    fn into_response_error(self) -> Error {
        self.into()
    }
}

impl IntoResponseError for Error {
    fn into_response_error(self) -> Error {
        self
    }
}
