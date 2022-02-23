use crate::http::{ByteStr, Headers, Request, Response, Status};
use crate::reject::Reject;

use std::{fmt, iter};

/// Types that be converted into an HTTP header.
pub trait IntoHeader {
    /// An iterator over the header values.
    ///
    /// This iterator is expected to yield at least
    /// one item.
    type Values: IntoIterator<Item = ByteStr>;

    /// Serializes the header name and values.
    fn into_header(self) -> (ByteStr, Self::Values);
}

impl<N, V> IntoHeader for (N, V)
where
    N: Into<ByteStr>,
    V: Into<ByteStr>,
{
    type Values = iter::Once<ByteStr>;

    fn into_header(self) -> (ByteStr, Self::Values) {
        (self.0.into(), iter::once(self.1.into()))
    }
}

/// A typed HTTP header.
pub trait Header<'a>: Sized {
    /// Error that can occur when extracting this header.
    type Rejection: Reject;

    /// Extracts the header from the header map.
    fn extract(headers: &Headers) -> Result<Self, Self::Rejection>;
}

/// The rejection returned when an expected header
/// is not found.
///
/// This will respond with 400 Bad Request.
#[derive(Debug)]
pub struct MissingHeader {
    name: ByteStr,
}

impl fmt::Display for MissingHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected header '{}'", self.name)
    }
}

impl Reject for MissingHeader {
    fn reject(self, _: &Request) -> Response {
        Response::from(Status::BadRequest)
    }
}
