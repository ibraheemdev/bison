use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Application state that can be registered at startup. This includes singletons as well as
/// request-scoped services.
#[derive(Clone)]
pub struct State {
    map: Arc<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            map: Arc::default(),
        }
    }

    pub(crate) fn insert<T>(&mut self, val: T)
    where
        T: Send + Sync + 'static,
    {
        Arc::get_mut(&mut self.map)
            .unwrap_or_else(|| panic!("cannot insert app state while serving requests"))
            .insert(TypeId::of::<T>(), Box::new(val));
    }

    /// Retrieve an object previously registered as state.
    pub fn get<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref())
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State").finish()
    }
}
