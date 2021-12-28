use super::NoArgument;
use crate::error::ResponseError;
use crate::http::{Body, Request, Response, ResponseBuilder, StatusCode};
use crate::state::{self, State};

use std::fmt;

/// Extracts application state from a request.
///
/// Application state can be injected with [`Bison::inject`](crate::Bison::inject).
pub fn state<'req, T>(req: &'req Request, _: NoArgument) -> Result<&'req T, StateError>
where
    T: State,
{
    req.extensions()
        .get::<state::Map>()
        .unwrap()
        .get::<T>()
        .ok_or(StateError {
            ty: std::any::type_name::<T>(),
        })
}

/// The error returned by [`extract::state`](state) if extraction fails.
///
/// Returns a 400 response if used as a [`ResponseError`].
#[derive(Debug)]
pub struct StateError {
    ty: &'static str,
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No state injected of type `{}`", self.ty)
    }
}

impl ResponseError for StateError {
    fn respond(&self) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .unwrap()
    }
}
