use crate::bison::State;
use crate::{Error, Request, Response, ResponseBuilder, ResponseError, SendBound};

use std::convert::Infallible;
use std::fmt;
use std::future::{ready, Future, Ready};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};
use std::str::FromStr;

/// Represents type that has context about the current HTTP request.
/// ```rust
/// #[derive(HasContext)]
/// struct CurrentUser {
///     #[header("Auth")]
///     token: String,
///     #[state]
///     db: Database
/// }
/// ```
pub trait HasContext<S>: Sized + SendBound {
    /// An error that can occur during constructing this type.
    type ConstructionError: Into<Error>;

    /// The future returned by [`construct`](Self::construct).
    type ConstructionFuture: Future<Output = Result<Self, Self::ConstructionError>> + SendBound;

    /// Construct this type from an HTTP request.
    fn extract(request: Request<S>) -> Self::ConstructionFuture;
}

impl<S> HasContext<S> for Request<S>
where
    S: State,
{
    type ConstructionError = Infallible;
    type ConstructionFuture = Ready<Result<Request<S>, Infallible>>;

    fn extract(request: Request<S>) -> Ready<Result<Request<S>, Infallible>> {
        ready(Ok(request))
    }
}

pub trait ParamContext: Sized {
    type Error: Into<Error>;

    fn extract(param: &str) -> Result<Self, Self::Error>;
}

impl ParamContext for String {
    type Error = Infallible;

    fn extract(param: &str) -> Result<Self, Self::Error> {
        Ok(param.to_string())
    }
}

macro_rules! param_from_str {
    ($($ty:ident),*) => {
        $(
            impl ParamContext for $ty {
                type Error = ParamParseError<<$ty as FromStr>::Err>;

                fn extract(param: &str) -> Result<Self, Self::Error> {
                    param.parse().map_err(|err| ParamParseError { param: param.to_owned(), err })
                }
            }
        )*
    };
}

param_from_str! {
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64,
    IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, SocketAddr, bool,
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize,
    NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize
}

#[doc(hidden)]
pub struct ParamParseError<E> {
    param: String,
    err: E,
}

impl<E> fmt::Debug for ParamParseError<E>
where
    E: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error while parsing route parameter '{}': {:?}",
            &self.param, &self.err
        )
    }
}

impl<E> ResponseError for ParamParseError<E>
where
    E: fmt::Debug + 'static,
{
    fn respond(&mut self) -> Response {
        Response::not_found()
    }
}
