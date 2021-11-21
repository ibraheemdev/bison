use crate::http::Response;

use std::convert::Infallible;
use std::fmt::{self, Debug};

pub trait ResponseError: Debug {
    fn respond(&mut self) -> Response;
}

impl ResponseError for Response {
    fn respond(&mut self) -> Response {
        std::mem::take(self)
    }
}

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

impl<'req> Error<'req> {
    pub fn new(err: impl ResponseError + 'req) -> Self {
        Self {
            inner: Box::new(err),
        }
    }

    pub fn as_mut(&mut self) -> &mut (impl ResponseError + 'req) {
        &mut self.inner
    }

    pub fn into_response_error(self) -> impl ResponseError + 'req {
        self.inner
    }
}

// We can't implement From<E> *for* Error and ResponseError for Error, but we still want users to be
// able to return the boxed Error from endpoints, and use the ? operator to propagate reponse
// errors. To accomplish this we have:
// ```rust
// trait Endpoint/Wrap/Other {
//     type Error: IntoResponseError;
// }
//
// impl<E: ResponseError> From<E> for Error { ... }
// ```
//
// And we lose the `impl ResponseError for Error`, which isn't that big of a deal because the inner
// error is still exposed.
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

pub trait IntoResponseError<'req>: Debug {
    fn into_response_error(self) -> Error<'req>;
}

impl<'req, E> IntoResponseError<'req> for E
where
    E: ResponseError + Debug + 'req,
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
