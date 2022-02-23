use crate::bounded::{BoxError, BoxStream, Send, Sync};

use std::error::Error as StdError;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{fmt, mem};

pub use bytes::Bytes;
pub use http::{header, Extensions, HeaderValue, Method, StatusCode};

use futures_core::Stream;

/// Respresents the body of an HTTP message.
pub struct Body { field1: BodyKind }

enum BodyKind {
    Stream(BoxStream<'static, Result<Bytes, BoxError>>),
    Once(Bytes),
    Empty,
    Taken,
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

        Body { field1: BodyKind::Stream(Box::pin(MapErr(stream))) }
    }

    pub fn try_clone(&self) -> Option<Body> {
        let kind = match self.field1 {
            BodyKind::Stream(_) => return None,
            BodyKind::Once(ref b) => BodyKind::Once(b.clone()),
            BodyKind::Empty => BodyKind::Empty,
            BodyKind::Taken => BodyKind::Taken,
        };

        Some(Body { field1: kind })
    }

    /// Create a body directly from bytes.
    pub fn once(bytes: impl Into<Bytes>) -> Self {
        Body { field1: BodyKind::Once(bytes.into()) }
    }

    /// Create an empty `Body`.
    pub fn empty() -> Self {
        Body { field1: BodyKind::Empty }
    }

    pub fn take(&self) -> Option<Body> {
        match mem::replace(&mut self.field1, BodyKind::Taken) {
            BodyKind::Taken => None,
            b => Some(Body { field1: b }),
        }
    }

    pub async fn chunk(&self) -> Option<Result<Bytes, BoxError>> {
        let mut this = self;
        crate::util::poll_fn(|cx| Pin::new(&mut this).poll_next(cx)).await
    }
}

impl Stream for &Body {
    type Item = Result<Bytes, BoxError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut self.field1 {
            BodyKind::Stream(stream) => stream.as_mut().poll_next(cx),
            BodyKind::Once(bytes) => {
                let bytes = mem::take(bytes);
                self.field1 = BodyKind::Empty;
                Some(Ok(bytes)).into()
            }
            BodyKind::Empty | BodyKind::Taken => None.into(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &*self.field1.borrow_mut() {
            BodyKind::Stream(stream) => stream.size_hint(),
            BodyKind::Once(bytes) => (bytes.len(), Some(bytes.len())),
            BodyKind::Empty | BodyKind::Taken => (0, Some(0)),
        }
    }
}

impl Stream for Body {
    type Item = Result<Bytes, BoxError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        <&Body>::poll_next(Pin::new(&mut &*self), cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        <&Body>::size_hint(&self)
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
