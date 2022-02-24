use super::Bytes;

use std::borrow::Borrow;
use std::convert::Infallible;
use std::error::Error;
use std::fmt;
use std::net::*;
use std::num::*;

/// A UTF-8 encoded string stored as [`Bytes`].
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteStr(Bytes);

impl ByteStr {
    pub fn new(str: impl Into<ByteStr>) -> ByteStr {
        str.into()
    }

    pub fn from_static(str: &'static str) -> ByteStr {
        ByteStr(Bytes::from_static(str.as_bytes()))
    }

    pub fn as_str(&self) -> &str {
        self
    }

    pub fn into_bytes(self) -> Bytes {
        self.0
    }
}

impl std::ops::Deref for ByteStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // SAFETY: always valid UTF-8
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl fmt::Debug for ByteStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl fmt::Display for ByteStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AsRef<str> for ByteStr {
    fn as_ref(&self) -> &str {
        &*self
    }
}

impl Borrow<str> for ByteStr {
    fn borrow(&self) -> &str {
        &*self
    }
}

impl PartialEq<str> for ByteStr {
    fn eq(&self, other: &str) -> bool {
        &*self == other
    }
}

impl PartialEq<ByteStr> for str {
    fn eq(&self, other: &ByteStr) -> bool {
        self == &*other
    }
}

impl From<String> for ByteStr {
    fn from(string: String) -> Self {
        ByteStr(string.into())
    }
}

impl From<&str> for ByteStr {
    fn from(str: &str) -> Self {
        ByteStr(str.to_owned().into())
    }
}

/// Parse a value from a [`ByteStr`].
///
/// This trait is automatically implemented for common types
/// such as integers and bools, as well as `&str` and `ByteStr`
/// itself.
pub trait FromByteStr<'a>: Sized {
    /// The associated error which can be returned from parsing.
    type Err: Error;

    /// Parses a string `s` to return a value of this type.
    fn from_byte_str(s: &'a ByteStr) -> Result<Self, Self::Err>;
}

impl<'a> FromByteStr<'a> for ByteStr {
    type Err = Infallible;

    fn from_byte_str(s: &'a ByteStr) -> Result<Self, Self::Err> {
        Ok(s.clone())
    }
}

impl<'a> FromByteStr<'a> for &'a ByteStr {
    type Err = Infallible;

    fn from_byte_str(s: &'a ByteStr) -> Result<Self, Self::Err> {
        Ok(s)
    }
}

impl<'a> FromByteStr<'a> for &'a str {
    type Err = Infallible;

    fn from_byte_str(s: &'a ByteStr) -> Result<Self, Self::Err> {
        Ok(s)
    }
}

parse! {
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64,
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize,
    NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
    IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, SocketAddr, bool
}

macro_rules! parse {
    ($($T:ty),+) => ($(
        impl<'a> FromByteStr<'a> for $T {
            type Err = <$T as std::str::FromStr>::Err;

            #[inline]
            fn from_byte_str(s: &'a ByteStr) -> Result<Self, Self::Err> {
                s.parse()
            }
        }
    )+)
}

pub(self) use parse;
