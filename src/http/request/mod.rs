mod param;
pub use param::{FromParam, ParamRejection};

use super::{Body, ByteStr, Headers, Method, Uri, Version};
use crate::state::AppState;

#[derive(Default)]
pub struct Request {
    /// The request's method
    pub method: Method,

    /// The request's URI
    pub uri: Uri,

    /// The request's version
    pub version: Version,

    /// The request's headers
    pub headers: Headers,

    /// The request body.
    pub body: Body,

    // Route Parameters
    pub(crate) params: Vec<(ByteStr, ByteStr)>,

    // Application state
    pub(crate) state: Option<AppState>,
}
