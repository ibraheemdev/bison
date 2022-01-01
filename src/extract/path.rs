use crate::bounded::BoxError;
use crate::extract::arg::ParamName;
use crate::http::{Body, Request, Response, ResponseBuilder, StatusCode};
use crate::Reject;

use std::convert::Infallible;
use std::fmt;
use std::net::*;
use std::num::*;
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
pub async fn path<T>(req: &Request, name: ParamName) -> Result<T, PathRejection>
where
    T: FromPath,
{
    let name = name.0;

    let param = req.param(name).ok_or(PathRejection {
        name: name.to_owned(),
        kind: PathRejectionKind::NotFound,
    })?;

    T::from_path(param).map_err(|e| PathRejection {
        kind: PathRejectionKind::FromPath(e.into()),
        name: name.to_owned(),
    })
}

/// A type that can be extracted from a URL path segment.
///
/// Types implementing this trait can be used with the [`path`]
/// extractor.
pub trait FromPath: Sized {
    /// Errors that can occur in [`from_path`](FromPath::from_path).
    type Error: Into<BoxError>;

    /// Extract the type from a path segment.
    fn from_path(path: &str) -> Result<Self, Self::Error>;
}

impl FromPath for String {
    type Error = Infallible;

    fn from_path(param: &str) -> Result<Self, Self::Error> {
        Ok(param.to_owned())
    }
}

macro_rules! from_path {
    ($($ty:ty),*) => ($(
        impl FromPath for $ty {
            type Error = <$ty as FromStr>::Err;

            fn from_path(path: &str) -> Result<Self, Self::Error> {
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

/// The error returned by [`extract::path`](path()) if extraction fails.
#[derive(Debug)]
pub struct PathRejection {
    name: String,
    kind: PathRejectionKind,
}

#[derive(Debug)]
enum PathRejectionKind {
    FromPath(BoxError),
    NotFound,
}

impl fmt::Display for PathRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            PathRejectionKind::FromPath(err) => {
                write!(f, "error extracting route param '{}': {}", self.name, err)
            }
            PathRejectionKind::NotFound => write!(f, "route param '{}' not found", self.name),
        }
    }
}

impl Reject for PathRejection {
    fn reject(self, _: &Request) -> Response {
        let status = match self.kind {
            PathRejectionKind::FromPath(_) => StatusCode::BAD_REQUEST,
            PathRejectionKind::NotFound => StatusCode::NOT_FOUND,
        };

        ResponseBuilder::new()
            .status(status)
            .body(Body::empty())
            .unwrap()
    }
}
