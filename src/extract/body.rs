use bytes::{Bytes, BytesMut};

use crate::bounded::{BoxError, Send, Sync};
use crate::extract::arg::DefaultArgument;
use crate::http::{header, Body, Request, Response, ResponseBuilder, StatusCode};
use crate::Reject;

use std::convert::Infallible;
use std::fmt;

/// Extract the request body directly into a collection.
///
/// This extractor can be used to read the request body into a `Vec<u8>`, `Bytes`,
/// or `String`. [`BodyConfig`] can be used to configure the extraction process.
pub async fn body<T>(req: &Request, config: BodyConfig) -> Result<T, BodyError<T::Error>>
where
    T: FromBytes,
{
    let body = req.body().take().ok_or(BodyError(BodyErrorKind::Taken))?;

    if req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|x| x.to_str().ok())
        .and_then(|x| x.parse::<usize>().ok())
        > Some(config.limit)
    {
        return Err(BodyError(BodyErrorKind::Overflow));
    }

    let mut buf = BytesMut::with_capacity(8192);

    loop {
        match body.chunk().await {
            Some(Err(e)) => return Err(BodyError(BodyErrorKind::Io(e))),
            Some(Ok(chunk)) => {
                if buf.len() + chunk.len() > config.limit {
                    return Err(BodyError(BodyErrorKind::Overflow));
                } else {
                    buf.extend_from_slice(&chunk);
                }
            }
            None => break,
        }
    }

    T::from_bytes(buf.freeze()).map_err(|err| BodyError(BodyErrorKind::Decode(err)))
}

/// A type that can be decoded from the raw bytes of the request body.
///
/// Types implementing this trait can be used with the [`body`]
/// extractor.
pub trait FromBytes: Sized {
    /// Errors that can occur when decoding the bytes.
    type Error: fmt::Debug + fmt::Display + Send + Sync;

    /// Decode the bytes of the request body.
    fn from_bytes(bytes: Bytes) -> Result<Self, Self::Error>;
}

impl FromBytes for Bytes {
    type Error = Infallible;

    fn from_bytes(bytes: Bytes) -> Result<Self, Self::Error> {
        Ok(bytes)
    }
}

impl FromBytes for Vec<u8> {
    type Error = Infallible;

    fn from_bytes(bytes: Bytes) -> Result<Self, Self::Error> {
        Ok(bytes.to_vec())
    }
}

impl FromBytes for String {
    type Error = std::string::FromUtf8Error;

    fn from_bytes(bytes: Bytes) -> Result<Self, Self::Error> {
        String::from_utf8(bytes.to_vec())
    }
}

/// Configuration for the [`body`] extractor.
pub struct BodyConfig {
    limit: usize,
}

impl BodyConfig {
    /// Create a [`BodyConfig`] instance and set maximum number of
    /// that can be streamed.
    ///
    /// By default the limit is 256Kb.
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }
}

impl DefaultArgument for BodyConfig {
    fn new(_: &'static str) -> Self {
        BodyConfig {
            limit: 262_144, // (~256kB)
        }
    }
}

/// The error returned by [`extract::body`](body) if extraction fails.
///
/// Returns a 400 response when used as a rejection.
#[derive(Debug)]
pub struct BodyError<E>(BodyErrorKind<E>);

#[derive(Debug)]
enum BodyErrorKind<E> {
    Taken,
    Overflow,
    Io(BoxError),
    Decode(E),
}

impl<E> fmt::Display for BodyError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            BodyErrorKind::Io(err) => {
                write!(f, "failed to read request body: {}", err)
            }
            BodyErrorKind::Overflow => {
                write!(f, "body larger than limit")
            }
            BodyErrorKind::Taken => {
                write!(f, "cannot have two body extractors for a single handler")
            }
            BodyErrorKind::Decode(err) => {
                write!(f, "failed to extract body from request: {}", err)
            }
        }
    }
}

impl<E> Reject for BodyError<E>
where
    E: fmt::Debug + fmt::Display + Send + Sync,
{
    fn reject(self: Box<Self>, _: &Request) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .unwrap()
    }
}
