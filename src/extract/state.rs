use crate::http::{Body, Request, Response, ResponseBuilder, StatusCode};
use crate::state::{self, State};
use crate::Reject;

use std::fmt;

/// Extracts application state from a request.
///
/// Application state can be injected with [`Bison::inject`](crate::Bison::inject).
pub async fn state<'req, T>(req: &'req Request, _: ()) -> Result<&'req T, StateRejection>
where
    T: State,
{
    req.extensions()
        .get::<state::Map>()
        .unwrap()
        .get::<T>()
        .ok_or(StateRejection {
            ty: std::any::type_name::<T>(),
        })
}

/// The error returned by [`extract::state`](state()) if extraction fails.
///
/// Returns a 400 response when used as a rejection.
#[derive(Debug)]
pub struct StateRejection {
    ty: &'static str,
}

impl fmt::Display for StateRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No state injected of type `{}`", self.ty)
    }
}

impl Reject for StateRejection {
    fn reject(self: Box<Self>, _: &Request) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .unwrap()
    }
}
