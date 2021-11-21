use std::sync::Arc;

pub trait State: Send + Sync + 'static {}
impl<T> State for T where T: Send + Sync + 'static {}

#[derive(Clone)]
pub struct Map(Arc<http::Extensions>);

impl Map {
    pub fn new() -> Self {
        Self(Default::default())
    }
}
