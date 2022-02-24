use super::header::{ContentType, IntoHeader};
use super::{Body, Bytes, Headers, Status, Version};

use std::borrow::Cow;

#[derive(Debug, Default)]
pub struct Response {
    /// The response's status
    pub status: Status,

    /// The response's version
    pub version: Version,

    /// The response's headers
    pub headers: Headers,

    /// The response body
    pub body: Body,
}

impl Response {
    /// Create a new HTTP response.
    pub fn new() -> Response {
        Response::default()
    }

    /// Create an HTTP response from a responder.
    ///
    /// See [`Respond`] for details.
    pub fn from(responder: impl Respond) -> Response {
        responder.respond()
    }

    /// Set the status of this response.
    pub fn status(mut self, status: Status) -> Response {
        self.status = status;
        self
    }

    /// Append a response header.
    pub fn header(mut self, header: impl IntoHeader) -> Response {
        self.headers.append(header);
        self
    }

    /// Set the HTTP version of this response.
    pub fn version(mut self, version: Version) -> Response {
        self.version = version;
        self
    }

    /// Set the response body.
    pub fn body(mut self, body: Body) -> Response {
        self.body = body;
        self
    }
}

/// A type that can be converted into an HTTP response.
pub trait Respond: Sized {
    /// Convert into an HTTP response.
    fn respond(self) -> Response;
}

impl Respond for () {
    fn respond(self) -> Response {
        Response::new()
    }
}

impl Respond for Body {
    fn respond(self) -> Response {
        Response::new().body(self)
    }
}

impl Respond for Status {
    fn respond(self) -> Response {
        Response::new().status(self)
    }
}

macro_rules! content {
    ($($ty:ty $(|> $into:ident)? => $content_type:expr),* $(,)?) => { $(
        impl Respond for $ty {
            fn respond_ok(self) -> Response {
                Response::new()
                    .header($content_type)
                    .body(Body::once(self $(.$into())?))
            }
        })*
    }
}

content! {
    Bytes => ContentType::OctetStream,
    Vec<u8> => ContentType::OctetStream,
    &'static [u8] => ContentType::OctetStream,
    Cow<'static, [u8]> |> into_owned => ContentType::OctetStream,
    String => ContentType::Text,
    &'static str => ContentType::Text,
    Cow<'static, str> |> into_owned => ContentType::Text,
}
