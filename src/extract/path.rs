use crate::bounded::{Send, Sync};
use crate::error::ResponseError;
use crate::extract::OptionalArgument;
use crate::http::{Body, Request, RequestExt, Response, ResponseBuilder, StatusCode};

use std::convert::Infallible;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};
use std::str::FromStr;

/// Extracts a route parameter from the request path.
///
/// ```
/// use bison::{Context, Response, Bison};
///
/// #[derive(Context)]
/// struct GetUser {
///     #[cx(path)] // #[cx(path = "id")]
///     id: usize
/// }
///
/// async fn get_user(cx: GetUser) -> Response {
///     log::info!("getting user with id: {}", cx.id);
///     // ...
///     # Response::default()
/// }
///
/// let bison = Bison::new().get("/user/:id", get_user);
/// ```
pub fn path<'req, T>(
    req: &'req Request,
    param: OptionalArgument<&'static str>,
) -> Result<T, PathError<T::Error>>
where
    T: FromPath<'req>,
{
    let name = param.value.unwrap_or(param.field_name);

    let param = req.param(name).ok_or(PathError {
        error: None,
        name: name.to_owned(),
    })?;

    T::from_path(param).map_err(|e| PathError {
        error: Some(e),
        name: name.to_owned(),
    })
}

/// The error returned by [`extract::path`](path()) if extraction fails.
///
/// Returns a 404 response if used as a [`ResponseError`].
#[derive(Debug)]
pub struct PathError<E> {
    error: Option<E>,
    name: String,
}

impl<E> fmt::Display for PathError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.error {
            Some(err) => write!(f, "error extracting param '{}': {}", self.name, err),
            None => write!(f, "param '{}' not found", self.name),
        }
    }
}

impl<E> ResponseError for PathError<E>
where
    E: fmt::Debug + fmt::Display + Send + Sync,
{
    fn respond(&self) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .unwrap()
    }
}

/// A type that can be extracted from a URL path segment.
///
/// Types implementing this trait can be used with the [`path`]
/// extractor.
pub trait FromPath<'req>: Sized {
    type Error: fmt::Debug + fmt::Display + Send + Sync;

    /// Extract the type from a path segment.
    fn from_path(param: &'req str) -> Result<Self, Self::Error>;
}

impl<'req> FromPath<'req> for &'req str {
    type Error = Infallible;

    fn from_path(param: &'req str) -> Result<Self, Self::Error> {
        Ok(param)
    }
}

impl<'req> FromPath<'req> for String {
    type Error = Infallible;

    fn from_path(param: &'req str) -> Result<Self, Self::Error> {
        Ok(param.to_owned())
    }
}

macro_rules! from_path {
    ($($ty:ty),*) => ($(
        impl<'req> FromPath<'req> for $ty {
            type Error = <$ty as FromStr>::Err;

            fn from_path(path: &'req str) -> Result<Self, Self::Error> {
                <$ty as FromStr>::from_str(path)
            }
        }
    )*)
}

from_path! {
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64,
    bool, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, SocketAddr,
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize,
    NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize
}
