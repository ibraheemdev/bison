use once_cell::sync::OnceCell;

use super::OptionalArgument;
use crate::error::ResponseError;
use crate::http::{Body, Request, Response, ResponseBuilder, StatusCode};

use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};
use std::str::FromStr;

/// Extracts a query parameter from the request.
///
/// # Examples
///
/// ```
/// use bison::Context;
/// use bison::http::{Response, ResponseBuilder, StatusCode};
///
/// #[derive(Context)]
/// struct Search<'req> {
///     #[cx(query)]
///     name: &'req str,
/// }
///
/// fn search(cx: Search<'_>) -> Response {
///     log::info!("searching for user with name: {}", cx.name);
///     // ...
///     # Default::default()
/// }
/// ```
pub fn query<'req, T>(
    req: &'req Request,
    param: OptionalArgument<&'static str>,
) -> Result<T, QueryError<T::Error>>
where
    T: FromQuery<'req>,
{
    let param = param.value.unwrap_or(param.field_name);

    let query = req
        .uri()
        .query()
        .ok_or(QueryError(QueryErrorKind::NotFound(param)))?;

    let map = req
        .extensions()
        .get::<CachedQuery>()
        .unwrap()
        .0
        .get_or_try_init(|| {
            serde_urlencoded::from_str::<HashMap<String, String>>(query)
                .map_err(|err| QueryError(QueryErrorKind::Deser(err)))
        })?;

    let raw = map
        .get(param)
        .ok_or(QueryError(QueryErrorKind::NotFound(param)))?;

    T::from_query(raw)
        .map_err(|err| QueryError(QueryErrorKind::FromStr(std::any::type_name::<T>(), err)))
}

#[derive(Default)]
pub(crate) struct CachedQuery(OnceCell<HashMap<String, String>>);

/// The error returned by [`extract::query`](query) if extraction fails.
///
/// Returns a 404 response if used as a [`ResponseError`].
#[derive(Debug)]
pub struct QueryError<E>(QueryErrorKind<E>);

#[derive(Debug)]
enum QueryErrorKind<E> {
    NotFound(&'static str),
    Deser(serde_urlencoded::de::Error),
    FromStr(&'static str, E),
}

impl<E> fmt::Display for QueryError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            QueryErrorKind::NotFound(name) => write!(f, "query parameter '{}' not found", name),
            QueryErrorKind::Deser(err) => {
                write!(f, "failed to deserialize query parameters: {}", err)
            }
            QueryErrorKind::FromStr(ty, error) => write!(
                f,
                "failed to deserialize `{}` from query parameter: {}",
                ty, error
            ),
        }
    }
}

impl<E> ResponseError for QueryError<E>
where
    E: fmt::Debug + fmt::Display + Send + Sync,
{
    fn respond(&self) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}

/// A type that can be extracted from a URL query parameter.
///
/// Types implementing this trait can be used with the [`query`]
/// extractor.
pub trait FromQuery<'req>: Sized {
    type Error: fmt::Debug + fmt::Display + Send + Sync;

    /// Extract the type from a query segment.
    fn from_query(param: &'req str) -> Result<Self, Self::Error>;
}

impl<'req> FromQuery<'req> for &'req str {
    type Error = Infallible;

    fn from_query(query: &'req str) -> Result<Self, Self::Error> {
        Ok(query)
    }
}

impl<'req> FromQuery<'req> for String {
    type Error = Infallible;

    fn from_query(query: &'req str) -> Result<Self, Self::Error> {
        Ok(query.to_owned())
    }
}

macro_rules! from_path {
    ($($ty:ty),*) => ($(
        impl<'req> FromQuery<'req> for $ty {
            type Error = <$ty as FromStr>::Err;

            fn from_query(query: &'req str) -> Result<Self, Self::Error> {
                <$ty as FromStr>::from_str(query)
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
