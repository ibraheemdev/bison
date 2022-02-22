use super::{Body, Headers, Status, Version};

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
