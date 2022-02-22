mod map;

pub use map::Headers;

mod common;
pub use common::ContentType;

use super::ByteStr;

use std::iter;

/// Types that represent an HTTP header.
pub trait IntoHeader {
    /// An iterator over the header values.
    ///
    /// This iterator is expected to yield at least
    /// one item.
    type Values: IntoIterator<Item = ByteStr>;

    /// Returns the name and values of the header.
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
