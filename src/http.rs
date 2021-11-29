use crate::bounded::{BoxError, BoxStream, Send, Sync};

use std::error::Error as StdError;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{fmt, mem};

pub use bytes::Bytes;
pub use http::{header, HeaderValue, Method, StatusCode};

use futures_core::Stream;

pub type Request = http::Request<Body>;
pub type ResponseBuilder = http::response::Builder;
pub type Response = http::Response<Body>;

pub struct Params(pub(crate) Vec<(String, String)>);

impl Params {
    pub fn get(&self, name: impl AsRef<str>) -> Option<&str> {
        let name = name.as_ref();
        self.0
            .iter()
            .find(|(x, _)| x == name)
            .map(|(_, val)| val.as_ref())
    }
}

/// Respresents the body of an HTTP message.
#[non_exhaustive]
pub enum Body {
    Stream(BoxStream<'static, Result<Bytes, BoxError>>),
    Once(Bytes),
    Empty,
}

impl Body {
    /// Create a `Body` from a stream of bytes.
    pub fn stream<S, E>(stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes, E>> + Send + Sync + 'static,
        E: StdError + Send + Sync + 'static,
    {
        pub struct MapErr<S>(S);

        impl<T, E, S> Stream for MapErr<S>
        where
            E: StdError + Send + Sync + 'static,
            S: Stream<Item = Result<T, E>>,
        {
            type Item = Result<T, BoxError>;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                unsafe { self.map_unchecked_mut(|s| &mut s.0) }
                    .poll_next(cx)
                    .map_err(|err| Box::new(err) as _)
            }
        }

        Self::Stream(Box::pin(MapErr(stream)))
    }

    /// Create a body directly from bytes.
    pub fn once(bytes: impl Into<Bytes>) -> Self {
        Self::Once(bytes.into())
    }

    /// Create an empty `Body`.
    pub fn empty() -> Self {
        Self::Empty
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Body").finish()
    }
}

impl Stream for Body {
    type Item = Result<Bytes, BoxError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut *self {
            Self::Stream(stream) => stream.as_mut().poll_next(cx),
            Self::Once(bytes) => {
                let bytes = mem::take(bytes);
                *self = Self::Empty;
                Some(Ok(bytes)).into()
            }
            Self::Empty => None.into(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &*self {
            Self::Stream(stream) => stream.size_hint(),
            Self::Once(bytes) => (bytes.len(), Some(bytes.len())),
            Self::Empty => (0, Some(0)),
        }
    }
}
