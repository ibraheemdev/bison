use crate::bounded::{BoxError, BoxStream, Send, Sync};
use crate::util::poll_fn;

use std::error::Error as StdError;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{fmt, mem};

pub use bytes::Bytes;
pub use http::{header, Extensions, HeaderValue, Method, StatusCode};

use futures_core::Stream;

/// The body of an HTTP request or response.
///
/// Data is streamed, yielding chunks of [`Bytes`].
///
/// ```rust
/// use bison::{Request, Response, Body};
///
/// fn handle(mut req: &mut Request) -> Response {
///     for chunk in req.body.chunk() {
///         println!("body chunk {:?}", chunk);
///     }
///
///     Response::new(Body::new("Hello World!"))
/// }
/// ```
pub struct Body {
    kind: BodyKind,
}

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

        Body {
            kind: BodyKind::Stream(Box::pin(MapErr(stream))),
        }
    }

    pub fn try_clone(&self) -> Option<Body> {
        let kind = match self.kind {
            BodyKind::Stream(_) => return None,
            BodyKind::Once(ref b) => BodyKind::Once(b.clone()),
            BodyKind::Empty => BodyKind::Empty,
        };

        Some(Body { kind })
    }

    /// Create a body directly from bytes.
    pub fn once(bytes: impl Into<Bytes>) -> Self {
        Body {
            kind: BodyKind::Once(bytes.into()),
        }
    }

    /// Create an empty `Body`.
    pub fn empty() -> Self {
        Body {
            kind: BodyKind::Empty,
        }
    }

    pub async fn chunk(&mut self) -> Option<Result<Bytes, BoxError>> {
        let mut this = self;
        poll_fn(|cx| Pin::new(&mut this).poll_next(cx)).await
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}

impl Stream for Body {
    type Item = Result<Bytes, BoxError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut self.kind {
            BodyKind::Stream(stream) => stream.as_mut().poll_next(cx),
            BodyKind::Once(bytes) => {
                let bytes = mem::take(bytes);
                self.kind = BodyKind::Empty;
                Some(Ok(bytes)).into()
            }
            BodyKind::Empty => None.into(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.kind {
            BodyKind::Stream(ref stream) => stream.size_hint(),
            BodyKind::Once(ref bytes) => (bytes.len(), Some(bytes.len())),
            BodyKind::Empty => (0, Some(0)),
        }
    }
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Body").finish()
    }
}
