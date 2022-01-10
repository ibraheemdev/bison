use crate::bounded::{Send, Sync};
use crate::http::Request;
use crate::Rejection;

use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::task::{self, Poll};

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
pub trait Context: Send + Sync + Sized + 'static {
    type Future: Future<Output = Result<Self, Rejection>> + Send;

    fn extract(req: Request) -> Self::Future;
}

impl Context for Request {
    type Future = Ready<Result<Self, Rejection>>;

    fn extract(req: Request) -> Self::Future {
        ready(Ok(req))
    }
}

impl Context for () {
    type Future = Ready<Result<Self, Rejection>>;

    fn extract(_: Request) -> Self::Future {
        ready(Ok(()))
    }
}

impl<T: Context> Context for (T,) {
    type Future = TupleFut<T::Future>;

    fn extract(req: Request) -> Self::Future {
        TupleFut {
            fut: T::extract(req),
        }
    }
}

pin_project_lite::pin_project! {
    pub struct TupleFut<F> { #[pin] fut: F }
}

impl<F, T, E> Future for TupleFut<F>
where
    F: Future<Output = Result<T, E>>,
{
    type Output = Result<(T,), E>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.project().fut.poll(cx).map_ok(|val| (val,))
    }
}
