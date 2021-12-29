use crate::extract::arg::{FieldName, ParamName};
use crate::extract::{path, query, FromPath, FromQuery};
use crate::http::{Body, Request, Response, ResponseBuilder, StatusCode};
use crate::Reject;

use std::fmt;

/// The default extractor.
///
/// This is the extractor used when no `#[cx(..)]` is given. It tries to extract
/// the type using the [`path`](path()) extractor, and then the [`query`](query()) extractor,
/// returning [`DefaultError`] if both fail.
pub fn default<'req, T>(req: &'req Request, field_name: FieldName) -> Result<T, DefaultError>
where
    T: FromPath<'req> + FromQuery<'req>,
{
    path(req, ParamName(field_name.as_str()))
        .or_else(|_| query(req, ParamName(field_name.as_str())))
        .map_err(|_| DefaultError {
            ty: std::any::type_name::<T>(),
        })
}

/// The error returned by [`extract::default`](`default`).
///
/// Returns a 404 response when used as a rejection.
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

impl Reject for DefaultError {
    fn reject(self: Box<Self>, _: &Request) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}
