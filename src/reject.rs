use crate::http::{Response, Status};
use crate::Respond;

use std::convert::Infallible;
use std::fmt::{self, Debug, Display};

/// An error capable rejecting a request with an HTTP error esponse.
pub trait Reject: Debug + Display {
    /// Reject the request with an HTTP error response.
    fn reject(self) -> Response;
}

impl Reject for Status {
    fn reject(self) -> Response {
        self.respond()
    }
}

impl Reject for Infallible {
    fn reject(self) -> Response {
        unsafe { std::hint::unreachable_unchecked() }
    }
}

/// A dynamically typed rejection.
pub struct Rejection {
    inner: Box<dyn BoxedReject>,
}

impl Rejection {
    /// Create a new `Rejection`.
    pub fn new<E>(err: E) -> Self
    where
        E: IntoRejection,
    {
        err.into_response_error()
    }

    /// Convert this error into an HTTP response.
    ///
    /// This method is analogous to [`Reject::reject`],
    /// which cannot be implemented directly due to
    /// coherence rules.
    pub fn reject(self) -> Response {
        self.inner.reject()
    }
}

trait BoxedReject: Reject {
    fn reject(self: Box<Self>) -> Response;
}

impl<T: Reject> BoxedReject for T {
    fn reject(self: Box<Self>) -> Response {
        Reject::reject(*self)
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
pub trait IntoRejection {
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
            fn reject(self) -> Response {
                self.0
            }
        }

        Impl(self).into()
    }
}
