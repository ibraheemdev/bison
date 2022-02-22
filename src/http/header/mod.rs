mod map;
pub use map::Headers;

mod common;
pub use common::ContentType;

use super::ByteStr;

pub struct Header {
    pub name: ByteStr,
    pub value: ByteStr,
}

pub trait IntoHeader {
    fn into_header(self) -> Header;
}

impl<N, V> IntoHeader for (N, V)
where
    N: Into<ByteStr>,
    V: Into<ByteStr>,
{
    fn into_header(self) -> Header {
        Header {
            name: self.0.into(),
            value: self.0.into(),
        }
    }
}
