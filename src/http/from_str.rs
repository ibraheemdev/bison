use std::convert::Infallible;
use std::error::Error;
use std::net::*;
use std::num::*;

/// Parse a value from a string.
///
/// The difference between this trait and [`std::str::FromStr`]
/// is that the lifetime `'a` allows values to borrow from the
/// input string.
///
/// This trait is automatically implemented for common types,
/// such as integers and bools.
pub trait FromStr<'a>: Sized {
    /// The associated error which can be returned from parsing.
    type Err: Error;

    /// Parses a string `s` to return a value of this type.
    fn from_str(s: &'a str) -> Result<Self, Self::Err>;
}

impl<'a> FromStr<'a> for &'a str {
    type Err = Infallible;

    fn from_str(s: &'a str) -> Result<Self, Self::Err> {
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
        impl<'a> FromStr<'a> for $T {
            type Err = <$T as std::str::FromStr>::Err;

            #[inline]
            fn from_str(s: &'a str) -> Result<Self, Self::Err> {
                s.parse()
            }
        }
    )+)
}

pub(self) use parse;
