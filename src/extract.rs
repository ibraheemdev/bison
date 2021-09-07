use crate::{bison::State, Error, HasContext};

use std::future::Future;

/// A type that is capable of extracting a specific value from it's context.
///
/// ```rust
/// use bison::{HasContext, ResponseError, Extractor};
/// use bison::error::Unauthorized;
///
/// #[derive(HasContext)]
/// struct CurrentUser {
///     #[header("Auth")]
///     token: String,
///     #[state]
///     db: Database
/// }
///
/// struct User { name: String }
///
/// impl Extractor for CurrentUser {
///     type Output = User;
///     type Error = Unauthorized;
///     type Future = BoxFuture<'static, Result<User, Unauthorized>>;
///
///     fn extract(self) -> Self::Future {
///         async { self.db.get_user(self.token).await.ok_or(Unauthorized) }
///     }
/// }
/// ```
///
/// Extractors can be used to derive extra context from a request:
/// ```rust
/// #[derive(HasContext)]
/// struct ResetPassword {
///     #[query("password")]
///     new_password: String,
///     #[extract(CurrentUser)]
///     user: User,
///     #[state]
///     database: Database
/// }
///
/// impl ResetPassword {
///     async fn handle(self) -> Result<Response, Infallible> {
///         self.database.update_user_password(user.id, new_password);
///         Ok(Response::success())
///     }
/// }
/// ```
pub trait Extract<S: State>: HasContext<S> {
    /// The type that is being extracted.
    type Output;

    /// An error that can occur during extraction.
    ///
    /// This is either a type implementing [`ResponseError`] or the boxed [`Error`]
    type Error: Into<Error>;

    /// The future returned by [`extract`](Self::extract).
    type Future: Future<Output = Result<Self::Output, Self::Error>>;

    /// Perform the extraction.
    fn extract(self) -> Self::Future;
}
