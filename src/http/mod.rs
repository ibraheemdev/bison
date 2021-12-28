mod body;
pub use body::Body;

pub(crate) mod ext;
pub use ext::{RequestExt, RequestBuilderExt};

pub use bytes::Bytes;
pub use http::{header, Extensions, HeaderValue, Method, StatusCode};

/// An HTTP request.
///
/// See [`http::Request`](http::Request) for details.
pub type Request = http::Request<Body>;

/// An HTTP response.
///
/// You can create a response with the [`new`](http::Response::new) method:
///
/// ```
/// # use astra::{Response, Body};
/// let response = Response::new(Body::new("Hello world!"));
/// ```
///
/// Or with a [`ResponseBuilder`]:
///
/// ```
/// # use astra::{ResponseBuilder, Body};
/// let response = ResponseBuilder::new()
///     .status(404)
///     .header("X-Custom-Foo", "Bar")
///     .body(Body::new("Page not found."))
///     .unwrap();
/// ```
///
/// See [`http::Response`](http::Response) for details.
pub type Response = http::Response<Body>;

/// A builder for an HTTP response.
///
/// ```
/// use astra::{ResponseBuilder, Body};
///
/// let response = ResponseBuilder::new()
///     .status(404)
///     .header("X-Custom-Foo", "Bar")
///     .body(Body::new("Page not found."))
///     .unwrap();
/// ```
///
/// See [`http::Response`](http::Response) and [`Body`] for details.
pub type ResponseBuilder = http::response::Builder;

/// A builder for an HTTP request.
///
/// This is useful for testing. See [`http::request::Builder`](http::request::Builder)
/// for details.
pub type RequestBuilder = http::request::Builder;
