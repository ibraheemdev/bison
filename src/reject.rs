//! HTTP error handling.

use crate::http::{Body, Request, Response, StatusCode};

use std::convert::Infallible;
use std::fmt::{self, Debug, Display};

/// An error capable rejecting a request with an HTTP error esponse.
pub trait Reject: Debug + Display + Send + Sync + 'static {
    /// Reject the request with an HTTP error esponse
    fn reject(self, req: &Request) -> Response;
}

impl Reject for Infallible {
    fn reject(self, _: &Request) -> Response {
        unreachable!()
    }
}

/// A dynamically typed rejection.
pub struct Rejection {
    inner: Box<dyn BoxedReject>,
}

impl Rejection {
    /// Create a new `AnyResponseError` from a given response error.
    pub fn new<E>(err: E) -> Self
    where
        E: IntoRejection,
    {
        err.into_response_error()
    }

    /// Convert this error into an HTTP response.
    ///
    /// This method is analogous to [`Reject::reject`],
    /// which cannot be implemented directly due to coherence rules.
    pub fn reject(self, req: &Request) -> Response {
        self.inner.reject(req)
    }
}

trait BoxedReject: Reject {
    fn reject(self: Box<Self>, _: &Request) -> Response;
}

impl<T: Reject> BoxedReject for T {
    fn reject(self: Box<Self>, req: &Request) -> Response {
        Reject::reject(*self, req)
    }
}

impl fmt::Debug for Rejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl fmt::Display for Rejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl<E> From<E> for Rejection
where
    E: Reject + 'static,
{
    fn from(err: E) -> Self {
        Self {
            inner: Box::new(err),
        }
    }
}

/// A type that can be converted into a [`Rejection`].
///
/// This trait allows [`Rejection`] and [`Response`]
/// to be used as rejections while not implementing
/// [`Reject`] directly. You shouldn't have to
/// worry about this trait, but it may show up
/// in error messages when [`Reject`] is not
/// implemented.
pub trait IntoRejection: Send + Sync {
    fn into_response_error(self) -> Rejection;
}

impl<E> IntoRejection for E
where
    E: Reject + 'static,
{
    fn into_response_error(self) -> Rejection {
        self.into()
    }
}

impl IntoRejection for Rejection {
    fn into_response_error(self) -> Rejection {
        self
    }
}

impl IntoRejection for Response {
    fn into_response_error(self) -> Rejection {
        struct Impl(Response);

        impl fmt::Debug for Impl {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }

        impl fmt::Display for Impl {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "status {:?}: {:?}", self.0.status, self.0.body)
            }
        }

        impl Reject for Impl {
            fn reject(self, _: &Request) -> Response {
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

impl Reject for NotFound {
    fn reject(self, _: &Request) -> Response {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
    }
}
