use crate::bounded::{Send, Sync};
use crate::http::Response;

use std::convert::Infallible;
use std::fmt::{self, Debug, Display};

/// An error capable of being transformed into an HTTP response.
pub trait ResponseError: Debug + Display + Send + Sync {
    /// Returns an HTTP response for this error.
    fn respond(&self) -> Response;
}

impl ResponseError for Infallible {
    fn respond(&self) -> Response {
        unreachable!()
    }
}

impl<'a> ResponseError for Box<dyn ResponseError + 'a> {
    fn respond(&self) -> Response {
        (&**self).respond()
    }
}

/// A dynamically typed response error.
pub struct AnyResponseError {
    inner: Box<dyn ResponseError>,
}

impl AnyResponseError {
    /// Create a new `AnyResponseError` from a given response error.
    pub fn new(err: impl ResponseError + 'static) -> Self {
        Self {
            inner: Box::new(err),
        }
    }

    /// Convert this error into an HTTP response.
    ///
    /// This method is analogous to [`ResponseError::response`],
    /// which cannot be implemented directly due to coherence rules.
    pub fn respond(&self) -> Response {
        self.inner.respond()
    }
}

impl fmt::Debug for AnyResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl fmt::Display for AnyResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl<E> From<E> for AnyResponseError
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
/// This trait is used in bounds as [`AnyResponseError`]
/// cannot implement [`ResponseError`] directly due
/// while retaining it's blanket `From` impl for use
/// with `?` operator, due to coherence rules.
pub trait IntoResponseError: Send {
    fn into_response_error(self) -> AnyResponseError;
}

impl<E> IntoResponseError for E
where
    E: ResponseError + 'static,
{
    fn into_response_error(self) -> AnyResponseError {
        self.into()
    }
}

impl IntoResponseError for AnyResponseError {
    fn into_response_error(self) -> AnyResponseError {
        self
    }
}
