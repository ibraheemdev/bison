use std::sync::Arc;

pub trait State: Send + Sync + 'static {}
impl<T> State for T where T: Send + Sync + 'static {}

#[derive(Clone)]
pub struct Map(Arc<http::Extensions>);

impl Map {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn get<T: State>(&self) -> Option<&T> {
        self.0.get::<T>()
    }

    pub fn insert<T: State>(self, state: T) -> Result<Self, ()> {
        let mut inner = Arc::try_unwrap(self.0).map_err(drop)?;
        inner.insert(state);
        Ok(Self(Arc::new(inner)))
    }
}
