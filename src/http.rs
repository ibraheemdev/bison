use crate::bounded::{BoxError, BoxStream, Send, Sync};
use crate::util::AtomicRefCell;

use std::error::Error as StdError;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{fmt, mem};

pub use bytes::Bytes;
pub use http::{header, Extensions, HeaderValue, Method, StatusCode};

use futures_core::Stream;

/// An HTTP request.
///
/// See [`http::Request`](http::Request) and [`Body`] for details.
pub type Request = http::Request<Body>;

/// An HTTP response.
///
/// You can create a response with the [`new`](hyper::Response::new) method:
///
/// ```
/// # use astra::{Response, Body};
/// let response = Response::new(Body::new("Hello world!"));
/// ```
///
/// Or with a [`ResponseBuilder`]:
///
/// ```
/// # use astra::{ResponseBuilder, Body};
/// let response = ResponseBuilder::new()
///     .status(404)
///     .header("X-Custom-Foo", "Bar")
///     .body(Body::new("Page not found."))
///     .unwrap();
/// ```
///
/// See [`http::Response`](http::Response) and [`Body`] for details.
pub type Response = http::Response<Body>;

/// A builder for an HTTP response.
///
/// ```
/// use astra::{ResponseBuilder, Body};
///
/// let response = ResponseBuilder::new()
///     .status(404)
///     .header("X-Custom-Foo", "Bar")
///     .body(Body::new("Page not found."))
///     .unwrap();
/// ```
///
/// See [`http::Response`](http::Response) and [`Body`] for details.
pub type ResponseBuilder = http::response::Builder;

/// Respresents the body of an HTTP message.
pub struct Body(AtomicRefCell<BodyKind>);

enum BodyKind {
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

        Body(AtomicRefCell::new(BodyKind::Stream(Box::pin(MapErr(
            stream,
        )))))
    }

    /// Create a body directly from bytes.
    pub fn once(bytes: impl Into<Bytes>) -> Self {
        Body(AtomicRefCell::new(BodyKind::Once(bytes.into())))
    }

    /// Create an empty `Body`.
    pub fn empty() -> Self {
        Body(AtomicRefCell::new(BodyKind::Empty))
    }

    pub fn take(&self) -> Body {
        Body(AtomicRefCell::new(mem::replace(
            &mut *self.0.borrow_mut(),
            BodyKind::Empty,
        )))
    }
}

impl Stream for &Body {
    type Item = Result<Bytes, BoxError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut inner = self.0.borrow_mut();

        match &mut *inner {
            BodyKind::Stream(stream) => stream.as_mut().poll_next(cx),
            BodyKind::Once(bytes) => {
                let bytes = mem::take(bytes);
                *inner = BodyKind::Empty;
                Some(Ok(bytes)).into()
            }
            BodyKind::Empty => None.into(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &*self.0.borrow_mut() {
            BodyKind::Stream(stream) => stream.size_hint(),
            BodyKind::Once(bytes) => (bytes.len(), Some(bytes.len())),
            BodyKind::Empty => (0, Some(0)),
        }
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

pub struct Params(pub(crate) Vec<(String, String)>);

impl Params {
    pub fn get(&self, name: impl AsRef<str>) -> Option<&str> {
        let name = name.as_ref();

        self.0
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, val)| val.as_ref())
    }
}
