use crate::{state, Request, State};

/// Extension methods for [`Request`].
pub trait RequestExt {
    /// Returns the route parameter with the given name.
    fn param(&self, name: impl AsRef<str>) -> Option<&str>;

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

    fn state<T>(&self) -> Option<&T>
    where
        T: State,
    {
        self.extensions().get::<state::Map>().unwrap().get()
    }
}

/// Extension methods for [`request::Builder`].
pub trait RequestBuilderExt {
    /// Adds a route parameter to the request.
    fn param(&mut self, name: impl Into<String>, value: impl Into<String>);
}

impl RequestBuilderExt for http::request::Builder {
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
