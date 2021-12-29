use bytes::{Bytes, BytesMut};

use crate::bounded::BoxError;
use crate::extract::arg::DefaultArgument;
use crate::http::{header, Body, Request, Response, ResponseBuilder, StatusCode};
use crate::Reject;

use std::convert::Infallible;
use std::fmt;

/// Extract the request body directly into a collection.
///
/// This extractor can be used to read the request body into a `Vec<u8>`, `Bytes`,
/// or `String`. [`BodyConfig`] can be used to configure the extraction process.
pub async fn body<T>(req: &Request, config: BodyConfig) -> Result<T, BodyRejection>
where
    T: FromBytes,
{
    let body = req
        .body()
        .take()
        .ok_or(BodyRejection(BodyErrorKind::Taken))?;

    if req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|x| x.to_str().ok())
        .and_then(|x| x.parse::<usize>().ok())
        > Some(config.limit)
    {
        return Err(BodyRejection(BodyErrorKind::Overflow));
    }

    let mut buf = BytesMut::with_capacity(8192);

    loop {
        match body.chunk().await {
            Some(Err(e)) => return Err(BodyRejection(BodyErrorKind::Io(e))),
            Some(Ok(chunk)) => {
                if buf.len() + chunk.len() > config.limit {
                    return Err(BodyRejection(BodyErrorKind::Overflow));
                } else {
                    buf.extend_from_slice(&chunk);
                }
            }
            None => break,
        }
    }

    T::from_bytes(buf.freeze()).map_err(|err| BodyRejection(BodyErrorKind::Decode(err.into())))
}

/// A type that can be decoded from the raw bytes of the request body.
///
/// Types implementing this trait can be used with the [`body`]
/// extractor.
pub trait FromBytes: Sized {
    /// Errors that can occur when decoding the bytes.
    type Error: Into<BoxError>;

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
    /// Create a [`BodyConfig`] instance.
    pub fn new() -> Self {
        Self {
            limit: 262_144, // (~256kB)
        }
    }

    /// Set maximum number of bytes that can be streamed.
    ///
    /// By default the limit is 256Kb.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

impl DefaultArgument for BodyConfig {
    fn new(_: &'static str) -> Self {
        Self::new()
    }
}

/// The error returned by [`extract::body`](body) if extraction fails.
#[derive(Debug)]
pub struct BodyRejection(BodyErrorKind);

#[derive(Debug)]
enum BodyErrorKind {
    Taken,
    Overflow,
    Io(BoxError),
    Decode(BoxError),
}

impl fmt::Display for BodyRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            BodyErrorKind::Io(err) => write!(f, "failed to read request body: {}", err),
            BodyErrorKind::Overflow => write!(f, "body larger than limit"),
            BodyErrorKind::Decode(err) => write!(f, "failed to extract body from request: {}", err),
            BodyErrorKind::Taken => {
                write!(f, "cannot have two body extractors for a single handler")
            }
        }
    }
}

impl Reject for BodyRejection {
    fn reject(self: Box<Self>, _: &Request) -> Response {
        let status = match self.0 {
            BodyErrorKind::Taken => StatusCode::INTERNAL_SERVER_ERROR,
            BodyErrorKind::Overflow => StatusCode::PAYLOAD_TOO_LARGE,
            BodyErrorKind::Io(_) | BodyErrorKind::Decode(_) => StatusCode::BAD_REQUEST,
        };

        ResponseBuilder::new()
            .status(status)
            .body(Body::empty())
            .unwrap()
    }
}
