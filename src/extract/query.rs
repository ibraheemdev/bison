use crate::bounded::BoxError;
use crate::extract::arg::ParamName;
use crate::http::{Body, Request, Response, ResponseBuilder, StatusCode};
use crate::Reject;

use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::net::*;
use std::num::*;
use std::str::FromStr;

use once_cell::sync::OnceCell;

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
///     #[cx(query)] // #[cx(query = "name")]
///     name: &'req str,
/// }
///
/// fn search(cx: Search<'_>) -> Response {
///     log::info!("searching for user with name: {}", cx.name);
///     // ...
///     # Default::default()
/// }
/// ```
pub async fn query<T>(req: &Request, name: ParamName) -> Result<T, QueryRejection>
where
    T: FromQuery,
{
    let error = QueryRejection::builder(name.0, std::any::type_name::<T>());

    let raw = req
        .query(name.0)
        .ok_or(error.kind(QueryRejectionKind::NotFound))?;

    T::from_query(raw).map_err(|err| error.kind(QueryRejectionKind::FromQuery(err.into())))
}

#[derive(Default)]
pub(crate) struct CachedQuery(OnceCell<HashMap<String, String>>);

/// A type that can be extracted from a URL query parameter.
///
/// Types implementing this trait can be used with the [`query`]
/// extractor.
pub trait FromQuery: Sized {
    /// Errors that can occur in [`from_query`](FromQuery::from_query).
    type Error: Into<BoxError>;

    /// Extract the type from a query segment.
    fn from_query(param: &str) -> Result<Self, Self::Error>;
}

impl FromQuery for String {
    type Error = Infallible;

    fn from_query(query: &str) -> Result<Self, Self::Error> {
        Ok(query.to_owned())
    }
}

macro_rules! from_path {
    ($($ty:ty),*) => ($(
        impl FromQuery for $ty {
            type Error = <$ty as FromStr>::Err;

            fn from_query(query: &str) -> Result<Self, Self::Error> {
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

/// The error returned by [`extract::query`](query) if extraction fails.
///
/// Returns a 400 response when used as a rejection.
#[derive(Debug)]
pub struct QueryRejection {
    name: &'static str,
    ty: &'static str,
    kind: QueryRejectionKind,
}

impl QueryRejection {
    pub fn builder(name: &'static str, ty: &'static str) -> Self {
        Self {
            name,
            ty,
            kind: QueryRejectionKind::NotFound,
        }
    }

    fn kind(&self, kind: QueryRejectionKind) -> Self {
        QueryRejection {
            name: self.name,
            ty: self.ty,
            kind,
        }
    }
}

#[derive(Debug)]
enum QueryRejectionKind {
    NotFound,
    FromQuery(BoxError),
}

impl fmt::Display for QueryRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            QueryRejectionKind::NotFound => write!(f, "query parameter '{}' not found", self.name),
            QueryRejectionKind::FromQuery(error) => write!(
                f,
                "failed to deserialize `{}` from query parameter: {}",
                self.ty, error
            ),
        }
    }
}

impl Reject for QueryRejection {
    fn reject(self, _: &Request) -> Response {
        let status = match self.kind {
            QueryRejectionKind::FromQuery(_) => StatusCode::BAD_REQUEST,
            QueryRejectionKind::NotFound => StatusCode::NOT_FOUND,
        };

        ResponseBuilder::new()
            .status(status)
            .body(Body::empty())
            .unwrap()
    }
}
