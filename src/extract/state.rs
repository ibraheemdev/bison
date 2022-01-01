use crate::http::{Body, Request, Response, ResponseBuilder, StatusCode};
use crate::state::State;
use crate::Reject;

use std::fmt;

/// Extracts application state from a request.
///
/// Application state can be injected with [`Bison::inject`](crate::Bison::inject).
pub async fn state<T>(req: &Request, _: ()) -> Result<T, StateRejection>
where
    T: State,
{
    req.state::<T>()
        .ok_or(StateRejection {
            ty: std::any::type_name::<T>(),
        })
        .map(T::clone)
}

/// The error returned by [`extract::state`](state()) if extraction fails.
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
    fn reject(self, _: &Request) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()
    }
}
