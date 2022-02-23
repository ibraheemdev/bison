use crate::bounded::Send;
use crate::{Handler, Request, Result};

/// Asynchronous HTTP middleware.
#[async_trait::async_trait]
pub trait Wrap<'req>: Send + Sync {
    /// Wrap this middleware around a handler.
    async fn wrap(&self, req: &'req mut Request, next: impl Handler<'req>) -> Result;
}
