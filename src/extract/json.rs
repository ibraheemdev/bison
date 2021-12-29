use crate::extract::arg::DefaultArgument;
use crate::extract::{body, BodyConfig, BodyRejection};
use crate::http::{header, Body, Bytes, Request, ResponseBuilder, StatusCode};
use crate::{Reject, Response};

use serde::de::DeserializeOwned;

use std::fmt;

/// Deserialize the given type as JSON from the request body.
///
/// [`JsonConfig`] can be used to configure the extraction process
pub async fn json<T>(req: &Request, config: JsonConfig) -> Result<T, JsonRejection>
where
    T: DeserializeOwned,
{
    if !is_json(req) {
        return Err(JsonRejection(JsonRejectionKind::ContentType));
    }

    let body: Bytes = body(req, BodyConfig::new().limit(config.limit))
        .await
        .map_err(|err| JsonRejection(JsonRejectionKind::Body(err)))?;

    serde_json::from_slice(&body).map_err(|err| JsonRejection(JsonRejectionKind::Deser(err)))
}

/// Configuration for the [`json`] extractor.
pub struct JsonConfig {
    limit: usize,
}

impl JsonConfig {
    /// Create a [`JsonConfig`] instance.
    pub fn new() -> Self {
        Self {
            limit: 2_097_152, // (~2mb)
        }
    }

    /// Set maximum number of bytes that can be streamed.
    ///
    /// By default the limit is 2mb.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

impl DefaultArgument for JsonConfig {
    fn new(_: &'static str) -> Self {
        Self::new()
    }
}

fn is_json(req: &Request) -> bool {
    let mime = || {
        req.headers()
            .get(header::CONTENT_TYPE)?
            .to_str()
            .ok()?
            .parse::<mime::Mime>()
            .ok()
    };

    match mime() {
        Some(mime) => mime.subtype() == mime::JSON || mime.suffix() == Some(mime::JSON),
        None => false,
    }
}

#[derive(Debug)]
pub struct JsonRejection(JsonRejectionKind);

#[derive(Debug)]
enum JsonRejectionKind {
    ContentType,
    Body(BodyRejection),
    Deser(serde_json::Error),
}

impl fmt::Display for JsonRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            JsonRejectionKind::ContentType => write!(f, "expected content-type application/json"),
            JsonRejectionKind::Body(err) => write!(f, "failed to read body: {}", err),
            JsonRejectionKind::Deser(err) => write!(f, "failed to deserialize body: {}", err),
        }
    }
}

impl Reject for JsonRejection {
    fn reject(self: Box<Self>, req: &Request) -> Response {
        let status = match self.0 {
            JsonRejectionKind::Body(err) => return Box::new(err).reject(req), // TODO
            JsonRejectionKind::ContentType | JsonRejectionKind::Deser(_) => StatusCode::BAD_REQUEST,
        };

        ResponseBuilder::new()
            .status(status)
            .body(Body::empty())
            .unwrap()
    }
}
