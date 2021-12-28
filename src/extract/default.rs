use crate::error::ResponseError;
use crate::extract::{path, query, FromPath, FromQuery, NoArgument};
use crate::http::{Body, Request, Response, ResponseBuilder, StatusCode};

use std::fmt;

use super::OptionalArgument;

/// The default extractor.
///
/// This is the extractor used when no `#[cx(..)]` is given. It tries to extract
/// the type using the [`path`](path()) extractor, and then the [`query`](query()) extractor,
/// returning [`DefaultError`] if both fail.
pub fn default<'req, T>(req: &'req Request, arg: NoArgument) -> Result<T, DefaultError>
where
    T: FromPath<'req> + FromQuery<'req>,
{
    let arg = OptionalArgument::from(arg);

    path(req, arg.clone())
        .or_else(|_| query(req, arg))
        .map_err(|_| DefaultError {
            ty: std::any::type_name::<T>(),
        })
}

/// The error returned by [`extract::default`](`default`).
///
/// Returns a 404 response if used as a [`ResponseError`].
#[derive(Debug)]
pub struct DefaultError {
    ty: &'static str,
}

impl fmt::Display for DefaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to extract `{}` from route parameter or query string",
            self.ty
        )
    }
}

impl ResponseError for DefaultError {
    fn respond(self: Box<Self>) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}
