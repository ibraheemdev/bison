use std::fmt;
use std::io;

use bytes::Bytes;

/// The streaming body of an HTTP request or response.
///
/// Data is streamed by iterating over the body, which
/// yields chunks as [`Bytes`].
///
/// ```rust
/// use bison::{Request, Response, Body};
///
/// fn handle(mut req: Request) -> Response {
///     for chunk in req.body_mut() {
///         println!("body chunk {:?}", chunk);
///     }
///
///     Response::new(Body::new("Hello World!"))
/// }
/// ```
pub struct Body(astra::Body);

impl Body {
    /// Create a body from a string or bytes.
    ///
    /// ```rust
    /// # use bison::Body;
    /// let string = Body::new("Hello world!");
    /// let bytes = Body::new(vec![0, 1, 0, 1, 0]);
    /// ```
    pub fn new(data: impl Into<Bytes>) -> Body {
        Body(astra::Body::from(data.into()))
    }

    /// Create an empty body.
    pub fn empty() -> Body {
        Body(astra::Body::empty())
    }

    /// Create a body from an implementor of [`io::Read`].
    ///
    /// ```rust
    /// use bison::{Request, Response, ResponseBuilder, Body};
    /// use std::fs::File;
    ///
    /// fn handle(_request: Request) -> Response {
    ///     let file = File::open("index.html").unwrap();
    ///
    ///     ResponseBuilder::builder()
    ///         .header("Content-Type", "text/html")
    ///         .body(Body::wrap_reader(file))
    ///         .unwrap()
    /// }
    /// ```
    pub fn stream<R>(source: R) -> Body
    where
        R: io::Read + Send + 'static,
    {
        Body(astra::Body::wrap_reader(source))
    }
}

impl<T> From<T> for Body
where
    Bytes: From<T>,
{
    fn from(data: T) -> Body {
        Body::new(data)
    }
}

impl Iterator for Body {
    type Item = io::Result<Bytes>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}
