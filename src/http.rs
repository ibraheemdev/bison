use crate::bison::State;
use crate::send::{BoxError, BoxStream, SendBound};
use crate::Error;

use std::error::Error as StdError;
use std::fmt;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_util::stream::{Stream, TryStreamExt};
pub use http::{header, request, HeaderValue, Method, StatusCode};

pub struct Request<S> {
    pub params: Vec<(String, String)>,
    pub inner: http::Request<Body>,
    pub state: S,
}

impl<S> Request<S>
where
    S: State,
{
    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn param(&self, name: &str) -> Option<&str> {
        self.params
            .iter()
            .find_map(|(k, v)| (k == name).then(|| v.as_str()))
    }
}

impl<S> std::ops::Deref for Request<S>
where
    S: State,
{
    // TODO: inherent methods
    type Target = http::Request<Body>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S> std::ops::DerefMut for Request<S>
where
    S: State,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub type Response = http::Response<Body>;

pub trait ResponseBuilder {
    type Builder;

    // TODO: avoid allocating &'static str
    fn text(text: impl Into<String>) -> Self;
    fn not_found() -> Self;
    fn builder() -> Self::Builder;
}

impl ResponseBuilder for Response {
    type Builder = http::response::Builder;

    fn not_found() -> Self {
        Self::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }

    fn text(text: impl Into<String>) -> Self {
        Response::new(Body::Once(Bytes::from(text.into())))
    }

    fn builder() -> Self::Builder {
        http::Response::<()>::builder()
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
        S: Stream<Item = Result<Bytes, E>> + SendBound + 'static,
        E: StdError + SendBound + 'static,
    {
        Self::Stream(Box::pin(stream.map_err(|e| Box::new(e) as _)))
    }

    /// Create a body directly from bytes.
    pub fn once(bytes: Bytes) -> Self {
        Self::Once(bytes)
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

mod _priv {
    use super::*;

    pub trait Sealed {}
    impl<E> Sealed for Result<Response, E> where E: Into<Error> {}
    impl Sealed for Response {}
}

pub trait IntoResponse: _priv::Sealed {
    fn into_response(self) -> Result<Response, Error>;
}

impl<E> IntoResponse for Result<Response, E>
where
    E: Into<Error>,
{
    fn into_response(self) -> Result<Response, Error> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(err.into()),
        }
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Result<Response, Error> {
        Ok(self)
    }
}
