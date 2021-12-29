use std::any::{Any, TypeId};
use std::sync::atomic::{AtomicUsize, Ordering};

use once_cell::sync::OnceCell;

use super::{RequestBuilder, ResponseBuilder};
use crate::{state, Request, Response, State};

/// Extension methods for [`Request`].
pub trait RequestExt {
    /// Returns the route parameter with the given name.
    fn param(&self, name: impl AsRef<str>) -> Option<&str>;

    /// Caches the provided value and returns a reference to it.
    ///
    /// Cached values can be later retrieved with [`RequestExt::cached`].
    fn cache<T>(&self, value: T) -> &T
    where
        T: State;

    /// Retrieves a value of type `T` if one has been previously [cached](RequestExt::cache).
    fn cached<T>(&self) -> Option<&T>
    where
        T: State;

    /// Rerieves application state from the request.
    ///
    /// Application state can be injected with [`Bison::inject`](crate::Bison::inject).
    fn state<T>(&self) -> Option<&T>
    where
        T: State;
}

impl RequestExt for Request {
    fn param(&self, name: impl AsRef<str>) -> Option<&str> {
        self.extensions()
            .get::<Params>()
            .and_then(|params| params.get(name))
    }

    fn cache<T>(&self, value: T) -> &T
    where
        T: State,
    {
        self.extensions().get::<Cache>().unwrap().insert(value)
    }

    fn cached<T>(&self) -> Option<&T>
    where
        T: State,
    {
        self.extensions().get::<Cache>().unwrap().get::<T>()
    }

    fn state<T>(&self) -> Option<&T>
    where
        T: State,
    {
        self.extensions().get::<state::Map>().unwrap().get()
    }
}

/// Extension methods for [`Response`].
pub trait ResponseExt {
    /// Attempt to clone the response.
    ///
    /// This method will return `None` if
    /// the body is a stream, which cannot
    /// be cloned.
    fn try_clone(&self) -> Option<Response>;
}

impl ResponseExt for Response {
    fn try_clone(&self) -> Option<Response> {
        self.body().try_clone().map(|body| {
            let mut builder = ResponseBuilder::new();
            *builder.headers_mut().unwrap() = self.headers().clone();
            builder
                .status(self.status())
                .version(self.version())
                .body(body)
                .unwrap()
        })
    }
}

/// Extension methods for [`RequestBuilder`].
pub trait RequestBuilderExt {
    /// Adds a route parameter to the request.
    fn param(&mut self, name: impl Into<String>, value: impl Into<String>);
}

impl RequestBuilderExt for RequestBuilder {
    fn param(&mut self, name: impl Into<String>, value: impl Into<String>) {
        if let Some(params) = self.extensions_mut().unwrap().get_mut::<Params>() {
            params.0.push((name.into(), value.into()));
            return;
        }

        self.extensions_mut()
            .unwrap()
            .insert(Params(vec![(name.into(), value.into())]));
    }
}

pub(crate) struct Params(Vec<(String, String)>);

impl Params {
    pub fn get(&self, name: impl AsRef<str>) -> Option<&str> {
        let name = name.as_ref();

        self.0
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, val)| val.as_ref())
    }
}

impl FromIterator<(String, String)> for Params {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (String, String)>,
    {
        Params(iter.into_iter().collect())
    }
}

const ITEMS: usize = 8;

#[derive(Default)]
pub(crate) struct Cache {
    items: [OnceCell<(TypeId, Box<dyn Any + Send + Sync>)>; ITEMS],
    len: AtomicUsize,
}

impl Cache {
    fn get<T>(&self) -> Option<&T>
    where
        T: State,
    {
        let id = TypeId::of::<T>();

        self.items
            .iter()
            .take(self.len.load(Ordering::Acquire))
            // SAFETY: `self.len` are initialized
            .map(|cell| unsafe { cell.get_unchecked() })
            .find(|(i, _)| *i == id)
            .map(|(_, value)| value.downcast_ref().unwrap())
    }

    fn insert<T>(&self, value: T) -> &T
    where
        T: State,
    {
        let i = self.len.fetch_add(1, Ordering::AcqRel);
        self.items[i]
            .set((TypeId::of::<T>(), Box::new(value)))
            .unwrap();

        // SAFETY: we just set the value
        let (_, value) = unsafe { self.items[i].get_unchecked() };
        value.downcast_ref().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache() {
        let cache = Cache::default();
        assert_eq!(cache.get::<()>(), None);
        cache.insert(42_usize);
        cache.insert("42");
        cache.insert("42".to_string());
        assert_eq!(cache.get::<usize>(), Some(&42));
        assert_eq!(cache.get::<&str>(), Some(&"42"));
        assert_eq!(cache.get::<String>().map(String::as_str), Some("42"));
        assert_eq!(cache.get::<()>(), None);
    }
}
