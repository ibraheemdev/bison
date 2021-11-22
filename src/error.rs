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

pub struct Error<'r> {
    inner: Box<dyn ResponseError + 'r>,
}

impl<'r> fmt::Debug for Error<'r> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl<'r> fmt::Display for Error<'r> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<'r> Error<'r> {
    pub fn new(err: impl ResponseError + 'r) -> Self {
        Self {
            inner: Box::new(err),
        }
    }

    pub fn as_mut(&mut self) -> &mut (impl ResponseError + 'r) {
        &mut self.inner
    }

    pub fn into_response_error(self) -> impl ResponseError + 'r {
        self.inner
    }
}

impl<'r, E> From<E> for Error<'r>
where
    E: ResponseError + 'r,
{
    fn from(err: E) -> Self {
        Self {
            inner: Box::new(err),
        }
    }
}

pub trait IntoResponseError<'r> {
    fn into_response_error(self) -> Error<'r>;
}

impl<'r, E> IntoResponseError<'r> for E
where
    E: ResponseError + 'r,
{
    fn into_response_error(self) -> Error<'r> {
        self.into()
    }
}

impl<'r> IntoResponseError<'r> for Error<'r> {
    fn into_response_error(self) -> Error<'r> {
        self
    }
}
