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
pub async fn default<'req, T>(
    req: &'req Request,
    field_name: FieldName,
) -> Result<T, DefaultRejection>
where
    T: FromPath<'req> + FromQuery<'req>,
{
    if let Ok(val) = path(req, ParamName(field_name.as_str())).await {
        return Ok(val);
    }

    if let Ok(val) = query(req, ParamName(field_name.as_str())).await {
        return Ok(val);
    }

    Err(DefaultRejection {
        ty: std::any::type_name::<T>(),
    })
}

/// The error returned by [`extract::default`](`default`).
///
/// Returns a 400 response when used as a rejection.
#[derive(Debug)]
pub struct DefaultRejection {
    ty: &'static str,
}

impl fmt::Display for DefaultRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to extract `{}` from route parameter or query string",
            self.ty
        )
    }
}

impl Reject for DefaultRejection {
    fn reject(self, _: &Request) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .unwrap()
    }
}
