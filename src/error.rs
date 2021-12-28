use crate::bounded::{Send, Sync};
use crate::http::{Body, Response, ResponseBuilder, StatusCode};

use std::convert::Infallible;
use std::fmt::{self, Debug, Display};

/// An error capable of being transformed into an HTTP response.
pub trait ResponseError: Debug + Display + Send + Sync {
    /// Returns an HTTP response for this error.
    fn respond(self: Box<Self>) -> Response;
}

impl ResponseError for Infallible {
    fn respond(self: Box<Self>) -> Response {
        unreachable!()
    }
}

/// A dynamically typed response error.
pub struct Error {
    inner: Box<dyn ResponseError>,
}

impl Error {
    /// Create a new `AnyResponseError` from a given response error.
    pub fn new<E>(err: E) -> Self
    where
        E: IntoResponseError,
    {
        err.into_response_error()
    }

    /// Convert this error into an HTTP response.
    ///
    /// This method is analogous to [`ResponseError::respond`],
    /// which cannot be implemented directly due to coherence rules.
    pub fn respond(self) -> Response {
        self.inner.respond()
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
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

/// A type that can be converted into a response error.
///
/// This trait allows [`Error`] and [`Response`] to be
/// used as response errors while not implementing
/// [`ResponseError`] directly. You shouldn't have
/// to worry about this trait, but it may show up
/// in error messages when [`ResponseError`] is
/// not implemented.
pub trait IntoResponseError: Send + Sync {
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

impl IntoResponseError for Response {
    fn into_response_error(self) -> Error {
        struct Impl(Response);

        impl fmt::Debug for Impl {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }

        impl fmt::Display for Impl {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "status {:?}: {:?}", self.0.status(), self.0.body())
            }
        }

        impl ResponseError for Impl {
            fn respond(self: Box<Self>) -> Response {
                self.0
            }
        }

        Impl(self).into()
    }
}

/// A response error that returns a 404 not found response.
#[derive(Debug)]
pub struct NotFound;

impl fmt::Display for NotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "404 not found")
    }
}

impl ResponseError for NotFound {
    fn respond(self: Box<Self>) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}
