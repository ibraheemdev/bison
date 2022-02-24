use crate::bounded::BoxError;
use crate::http::Status;
use crate::{Reject, Request, Respond, Response};

use std::fmt;

use bytes::{Buf, Bytes};
use futures_core::Stream;

impl Request {
    /// Read the request body as bytes.
    ///
    /// `limit` indicates the maximum bytes that can be read
    /// before returning an error. The default limit is ~256 kb.
    pub async fn bytes(&mut self, limit: Option<usize>) -> Result<Bytes, BytesRejection> {
        let limit = limit.unwrap_or(256 * 1024); // ~256kb

        // TODO: check content-length first
        let check = |chunk: Bytes, total| {
            if chunk.len() + total > limit {
                Err(BytesRejection(BytesRejectionKind::ExceededLimit))
            } else {
                Ok(chunk)
            }
        };

        let first = match self.body.chunk().await {
            Some(chunk) => chunk
                .map_err(BytesRejection::io)
                .and_then(|b| check(b, 0))?,
            None => return Ok(Bytes::new()),
        };

        let second = match self.body.chunk().await {
            Some(chunk) => chunk
                .map_err(BytesRejection::io)
                .and_then(|b| check(b, first.len()))?,
            None => return Ok(first),
        };

        let cap = first.remaining() + second.remaining() + self.body.size_hint().0;
        let mut bytes = Vec::with_capacity(cap);

        bytes.extend_from_slice(&first);
        bytes.extend_from_slice(&second);

        while let Some(chunk) = self.body.chunk().await {
            let chunk = chunk
                .map_err(BytesRejection::io)
                .and_then(|b| check(b, bytes.len()))?;

            bytes.extend_from_slice(&chunk);
        }

        Ok(Bytes::from(bytes))
    }
}

/// Error returned by [`Request::bytes`].
///
/// This type will reject the request with [`Status::BadRequest`]
/// if an error occurs while reading the body, and [`Status::PayloadTooLarge`]
/// if the body size exceeds the configured limit.
#[derive(Debug)]
pub struct BytesRejection(BytesRejectionKind);

impl BytesRejection {
    fn io(err: BoxError) -> Self {
        BytesRejection(BytesRejectionKind::Io(err))
    }
}

#[derive(Debug)]
enum BytesRejectionKind {
    Io(BoxError),
    ExceededLimit,
}

impl Reject for BytesRejection {
    fn reject(self) -> Response {
        match self.0 {
            BytesRejectionKind::Io(_) => Status::BadRequest,
            BytesRejectionKind::ExceededLimit => Status::PayloadTooLarge,
        }
        .respond()
    }
}

impl fmt::Display for BytesRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            BytesRejectionKind::Io(ref e) => write!(f, "Failed to read body: {}", e),
            BytesRejectionKind::ExceededLimit => write!(f, "Body size exceeded limit"),
        }
    }
}
