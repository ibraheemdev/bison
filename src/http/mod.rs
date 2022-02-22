mod body;
mod bytestr;

pub mod header;

pub use body::Body;
pub use bytestr::ByteStr;
pub use header::Headers;

pub use bytes::Bytes;
pub use http::{Method, StatusCode, Uri, Version};

pub struct Request {
    /// The request's method
    pub method: Method,
    /// The request's URI
    pub uri: Uri,
    /// The request's version
    pub version: Version,
    /// The request's headers
    pub headers: Headers,
    /// The request's body.
    pub body: Body,
}

#[derive(Debug)]
pub struct Response {
    /// The response's status
    pub status: StatusCode,

    /// The response's version
    pub version: Version,

    /// The response's headers
    pub headers: Headers,

    /// The response's body
    pub body: Body,
}

impl Response {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder(Response::default())
    }
}

pub struct ResponseBuilder(Response);

impl ResponseBuilder {
    pub fn status(mut self, status: StatusCode) -> ResponseBuilder {
        self.0.status = status;
        self
    }

    pub fn version(mut self, version: Version) -> ResponseBuilder {
        self.0.version = version;
        self
    }

    pub fn header<H>(mut self, header: H) -> ResponseBuilder
    where
        H: header::IntoHeader,
    {
        self.0.headers.insert(header);
        self
    }

    pub fn body(mut self, body: Body) -> Response {
        self.0.body = body;
        self.0
    }
}

impl Default for Response {
    fn default() -> Response {
        Response {
            status: StatusCode::OK,
            version: Version::HTTP_11,
            headers: Headers::new(),
            body: Body::empty(),
        }
    }
}
