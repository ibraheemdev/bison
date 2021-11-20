use crate::http::{Response, ResponseBuilder};

use core::fmt;
use std::convert::Infallible;
use std::fmt::Debug;

/// An error that can be converted into an HTTP response.
///
/// The most common way to implement this trait is with the derive macro:
/// ```rust
/// #[derive(ResponseError)]
/// enum GetPostError {
///     #[status(404)]
///     #[json({ error: format!("Post with id '{}' not found", .id) })]
///     PostNotFound { id: usize },
///     #[status(400)]
///     #[json({ error: "Unauthorized" })]
///     UserNotFound,
/// }
/// ```
///
/// You can derive it on regular structs as well:
/// ```rust
/// #[derive(ResponseError)]
/// #[status(404)]
/// struct NotFound;
/// ```
pub trait ResponseError: Debug + 'static {
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

impl ResponseError for Box<dyn ResponseError> {
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

impl Error {
    pub fn new(err: impl ResponseError) -> Self {
        Self {
            inner: Box::new(err),
        }
    }

    pub fn as_ref(&self) -> &impl ResponseError {
        &self.inner
    }

    pub fn into_response_error(self) -> impl ResponseError {
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
impl<E> From<E> for Error
where
    E: ResponseError,
{
    fn from(err: E) -> Self {
        Self {
            inner: Box::new(err),
        }
    }
}

pub trait IntoResponseError: Debug {
    fn into_response_error(self) -> Error;
}

impl<E> IntoResponseError for E
where
    E: ResponseError + Debug,
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

pub struct ParamNotFound {
    _priv: (),
}

impl ParamNotFound {
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl fmt::Debug for ParamNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ParamNotFound").finish()
    }
}

impl ResponseError for ParamNotFound {
    fn respond(&mut self) -> Response {
        Response::not_found()
    }
}
