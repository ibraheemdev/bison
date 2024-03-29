use crate::extract::arg::DefaultArgument;
use crate::extract::{self, BodyConfig, BodyRejection};
use crate::http::{header, Body, Bytes, Request, ResponseBuilder, StatusCode};
use crate::{Reject, Response};

use serde::de::DeserializeOwned;

use std::fmt;

/// Deserialize the given type from a URL encoded form.
///
/// [`FormConfig`] can be used to configure the extraction process
pub async fn form<T>(req: &Request, config: FormConfig) -> Result<T, FormRejection>
where
    T: DeserializeOwned,
{
    if !is_url_encoded(req) {
        return Err(FormRejection(FormRejectionKind::ContentType));
    }

    let body = extract::body::<Bytes>(req, BodyConfig::new().limit(config.limit))
        .await
        .map_err(|err| FormRejection(FormRejectionKind::Body(err)))?;

    serde_urlencoded::from_bytes(&body).map_err(|err| FormRejection(FormRejectionKind::Deser(err)))
}

fn is_url_encoded(req: &Request) -> bool {
    match req.headers().get(header::CONTENT_TYPE).as_deref() {
        Some("application/x-www-form-urlencoded") => true,
        _ => false,
    }
}

/// Configuration for the [`form`] extractor.
pub struct FormConfig {
    limit: usize,
}

impl FormConfig {
    /// Create a [`FormConfig`] instance.
    pub fn new() -> Self {
        Self {
            limit: 16_384, // (~16kb)
        }
    }

    /// Set maximum number of bytes that can be streamed.
    ///
    /// By default the limit is 16kb.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

impl DefaultArgument for FormConfig {
    fn new(_: &'static str) -> Self {
        Self::new()
    }
}

/// The error returned by [`extract::form`](form) if extraction fails.
#[derive(Debug)]
pub struct FormRejection(FormRejectionKind);

#[derive(Debug)]
enum FormRejectionKind {
    ContentType,
    Body(BodyRejection),
    Deser(serde_urlencoded::de::Error),
}

impl fmt::Display for FormRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            FormRejectionKind::ContentType => {
                write!(f, "expected content-type application/x-www-form-urlencoded")
            }
            FormRejectionKind::Body(err) => write!(f, "failed to read body: {}", err),
            FormRejectionKind::Deser(err) => write!(f, "failed to deserialize body: {}", err),
        }
    }
}

impl Reject for FormRejection {
    fn reject(self, req: &Request) -> Response {
        let status = match self.0 {
            FormRejectionKind::Body(err) => return err.reject(req),
            FormRejectionKind::ContentType | FormRejectionKind::Deser(_) => StatusCode::BAD_REQUEST,
        };

        ResponseBuilder::new()
            .status(status)
            .body(Body::empty())
            .unwrap()
    }
}
