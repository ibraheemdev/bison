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
pub trait HasContext: Sized + SendBound {
    /// An error that can occur during constructing this type.
    type ConstructionError: ResponseError;

    /// The future returned by [`construct`](Self::construct).
    type ConstructionFuture: Future<Output = Result<Self, Self::ConstructionError>> + SendBound;

    /// Construct this type from an HTTP request.
    fn construct(request: Request) -> Self::ConstructionFuture;
}

impl HasContext for Request {
    type ConstructionError = Infallible;
    type ConstructionFuture = Ready<Result<Request, Infallible>>;

    fn construct(request: Request) -> Ready<Result<Request, Infallible>> {
        ready(Ok(request))
    }
}
