use crate::Rejection;

use std::fmt;
use std::marker::PhantomPinned;
use std::option::Option as StdOption;

/// A trait that allows transforming the result of an extractor.
pub trait Transform<T>: Sized {
    /// The success value of this transformationm generally `T`.
    type Ok;

    /// Perform the transformation.
    fn transform(result: Result<T, Rejection>) -> Result<Self, Rejection>;
}

impl<T> Transform<T> for T
where
    T: Unpin,
{
    type Ok = T;

    fn transform(result: Result<T, Rejection>) -> Result<Self, Rejection> {
        result
    }
}

/// A type that allows specialized transformers.
///
/// Specialized transformers like [`transform::Option`](Option)
/// and [`transform::Result`](Result) must embed this type
/// due to limitations of the rust type-system.
pub struct TransformSpecialization(PhantomPinned);

impl TransformSpecialization {
    /// Create a new instance of this type.
    pub fn new() -> Self {
        Self(PhantomPinned)
    }
}

/// Optional request context.
///
/// If you this type as context [`Option::None`]
/// will be returned instead of an HTTP response error.
/// This is useful when you want to handle errors manually
/// instead of the default error response.
///
/// This type exists instead of the standard `Option` type
/// due to limitations of the rust type-system.
///
/// # Examples
///
/// ```
/// use bison::Context;
/// use bison::http::{Response, ResponseBuilder, StatusCode};
///
/// #[derive(Context)]
/// struct Search<'req> {
///     query: Option<&'req str>
/// }
///
/// fn search(cx: Search<'_>) -> Response {
///     let query = match cx.query.into_inner() {
///         Ok(query) => query,
///         None => return ResponseBuilder::new()
///             .status(StatusCode::BAD_REQUEST)
///             .body("Must provide a search query".into())
///             .unwrap()
///     };
///
///     // ...
///     # Default::default()
/// }
/// ```
///
/// [`Option::None`]: std::option::Option::None
pub struct Optional<T> {
    value: StdOption<T>,
    _t: TransformSpecialization,
}

impl<T> std::ops::Deref for Optional<T> {
    type Target = StdOption<T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Optional<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> Optional<T> {
    /// Convert into a [`std::Option`](std::option::Option).
    pub fn into_inner(self) -> StdOption<T> {
        self.value
    }
}

impl<T> Transform<T> for Optional<T> {
    type Ok = T;

    fn transform(result: Result<T, Rejection>) -> Result<Self, Rejection> {
        Ok(Optional {
            value: result.ok(),
            _t: TransformSpecialization::new(),
        })
    }
}

impl<T> fmt::Debug for Optional<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.value, f)
    }
}
