use crate::bison::State;
use crate::{Request, ResponseError, SendBound};

use std::convert::Infallible;
use std::future::{ready, Future, Ready};

/// Represents type that has context about the current HTTP request.
/// ```rust
/// #[derive(HasContext)]
/// struct CurrentUser {
///     #[header("Auth")]
///     token: String,
///     #[state]
///     db: Database
/// }
/// ```
pub trait HasContext<S>: Sized + SendBound {
    /// An error that can occur during constructing this type.
    type ConstructionError: ResponseError;

    /// The future returned by [`construct`](Self::construct).
    type ConstructionFuture: Future<Output = Result<Self, Self::ConstructionError>> + SendBound;

    /// Construct this type from an HTTP request.
    fn construct(request: Request<S>) -> Self::ConstructionFuture;
}

impl<S> HasContext<S> for Request<S>
where
    S: State,
{
    type ConstructionError = Infallible;
    type ConstructionFuture = Ready<Result<Request<S>, Infallible>>;

    fn construct(request: Request<S>) -> Ready<Result<Request<S>, Infallible>> {
        ready(Ok(request))
    }
}
