use crate::bounded::{Send, Sync};
use crate::http::Request;
use crate::AnyResponseError;

use std::future::{ready, Future, Ready};

/// Context about an HTTP request.
///
/// This trait is usually implemented through it's derive macro:
/// ```
/// use bison::Context;
///
/// #[derive(Context)]
/// struct Hello<'req> {
///     name: &'req str,
/// }
///
/// async fn hello(cx: Hello<'_>) -> String {
///     format!("Hello {}!", cx.name)
/// }
/// ```
pub trait Context<'req>: Send + Sync + Sized {
    type Future: Future<Output = Result<Self, AnyResponseError>> + Send + 'req;

    fn extract(req: &'req Request) -> Self::Future;
}

/// A type with context about an HTTP request.
///
/// This trait is used to create the correct bounds
/// for handler functions. You shouldn't have to
/// worry about it, but it may show up in error messages.
pub trait WithContext<'req>: Send + Sync {
    type Context: Context<'req> + 'req;
}

impl<'req> Context<'req> for &'req Request {
    type Future = Ready<Result<Self, AnyResponseError>>;

    fn extract(req: &'req Request) -> Self::Future {
        ready(Ok(req))
    }
}

impl<'any, 'req> WithContext<'req> for &'any Request {
    type Context = &'req Request;
}

impl<'req> Context<'req> for () {
    type Future = Ready<Result<Self, AnyResponseError>>;

    fn extract(_: &'req Request) -> Self::Future {
        ready(Ok(()))
    }
}

impl<'req> WithContext<'req> for () {
    type Context = ();
}

impl<'req, T: WithContext<'req>> WithContext<'req> for (T,) {
    type Context = T::Context;
}
