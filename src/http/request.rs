use crate::bounded::{cfg_send, OnceCell, Rc, RefCell};
use crate::http::{Body, Headers};
use crate::state::{AppState, State};

use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::str::FromStr;
use std::sync::atomic::{AtomicU8, Ordering};

/// An HTTP method.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Method(u8);

impl Method {
    pub const GET: Method = Method(0);
    pub const PUT: Method = Method(1);
    pub const POST: Method = Method(2);
    pub const DELETE: Method = Method(3);
    pub const OPTIONS: Method = Method(4);
    pub const HEAD: Method = Method(5);
    pub const TRACE: Method = Method(6);
    pub const CONNECT: Method = Method(7);
    pub const PATCH: Method = Method(8);
}

/// An HTTP request.
#[derive(Clone)]
pub struct Request {
    shared: Rc<Shared>,
}

pub struct Shared {
    method: AtomicU8,
    uri: RefCell<Uri>,
    state: AppState,
    headers: Headers,
    cache: Cache,
    body: Body,
    route_params: Params,
    query_params: OnceCell<Params>,
}

impl Request {
    pub fn method(&self) -> Method {
        Method(self.shared.method.load(Ordering::Relaxed))
    }

    pub fn set_method(&self, method: Method) {
        self.shared.method.store(method.0, Ordering::Relaxed);
    }

    pub fn uri(&self) -> Uri {
        self.shared.uri.borrow_mut().clone()
    }

    pub fn set_uri(&self, uri: Uri) {
        *self.shared.uri.borrow_mut() = uri;
    }

    pub fn headers(&self) -> &Headers {
        &self.shared.headers
    }

    pub fn param(&self, name: &str) -> Option<&str> {
        self.shared.route_params.get(name)
    }

    pub fn query(&self, name: &str) -> Option<&str> {
        if let Some(query) = self.shared.uri.borrow_mut().query() {
            return self
                .shared
                .query_params
                .get_or_try_init(|| {
                    serde_urlencoded::from_str::<Vec<(String, String)>>(query).map(Params)
                })
                .ok()
                .and_then(|params| {
                    params
                        .0
                        .iter()
                        .find(|(n, _)| n == name)
                        .map(|(_, value)| value.as_str())
                });
        }

        None
    }

    pub fn body(&self) -> &Body {
        &self.shared.body
    }

    pub fn state<T>(&self) -> Option<&T>
    where
        T: State,
    {
        self.shared.state.get()
    }

    pub fn cached<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.shared.cache.get()
    }

    pub fn cache<T>(&self, value: T) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.shared.cache.set(value)
    }
}

impl Request {
    pub(crate) fn new(
        req: http::Request<Body>,
        state: AppState,
        route_params: Params,
    ) -> Option<Self> {
        let (req, body) = req.into_parts();

        Some(Request {
            shared: Rc::new(Shared {
                method: AtomicU8::new(Method::from_http(req.method)?.0),
                uri: RefCell::new(Uri(req.uri)),
                query_params: OnceCell::new(),
                headers: req.headers,
                cache: Cache::default(),
                route_params,
                body,
                state,
            }),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Uri(http::Uri);

impl Uri {
    pub fn query(&self) -> Option<&str> {
        self.0.query()
    }
}

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Uri {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Uri).map_err(drop)
    }
}

#[derive(Default)]
pub(crate) struct Params(Vec<(String, String)>);

impl Params {
    fn get(&self, name: impl AsRef<str>) -> Option<&str> {
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

type AnyMap = HashMap<TypeId, Box<dyn Any + Send + Sync>, BuildHasherDefault<Identity>>;

#[derive(Default)]
struct Cache {
    map: UnsafeCell<AnyMap>,
    guard: RefCell<()>,
}

impl Cache {
    fn get<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        let borrowed = self.guard.borrow_mut();
        // SAFETY: `borrowed` guarantees mutual exclusion,
        // and we can return a borrow because values are
        // boxed and so have a stable address
        let value = unsafe {
            (*self.map.get())
                .get(&TypeId::of::<T>())
                .map(|val| val.downcast_ref().unwrap())
        };
        drop(borrowed);
        value
    }

    fn set<T>(&self, value: T) -> bool
    where
        T: Send + Sync + 'static,
    {
        let borrowed = self.guard.borrow_mut();
        let id = TypeId::of::<T>();
        let map = self.map.get();
        // SAFETY: `borrowed` guarantees mutual exclusion
        let had = unsafe {
            let had = (*map).contains_key(&id);
            if !had {
                (*map).insert(id, Box::new(value));
            }
            had
        };
        drop(borrowed);
        had
    }
}

cfg_send! {
    // SAFETY: all accesses of the map are done through the RefCell
    unsafe impl Send for Cache where RefCell<()>: Send {}
    unsafe impl Sync for Cache where RefCell<()>: Sync {}
}

#[derive(Default)]
struct Identity(u64);

impl Hasher for Identity {
    fn write(&mut self, _: &[u8]) {
        unreachable!()
    }

    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

impl Method {
    fn from_http(method: http::Method) -> Option<Self> {
        let method = match method.as_str().len() {
            3 => match method {
                http::Method::GET => Method::GET,
                http::Method::PUT => Method::PUT,
                _ => return None,
            },
            4 => match method {
                http::Method::POST => Method::POST,
                http::Method::HEAD => Method::HEAD,
                _ => return None,
            },
            5 => match method {
                http::Method::PATCH => Method::PATCH,
                http::Method::TRACE => Method::TRACE,
                _ => return None,
            },
            6 => match method {
                http::Method::DELETE => Method::DELETE,
                _ => return None,
            },
            7 => match method {
                http::Method::OPTIONS => Method::OPTIONS,
                http::Method::CONNECT => Method::CONNECT,
                _ => return None,
            },
            _ => return None,
        };

        Some(method)
    }

    pub(crate) fn into_http(self) -> http::Method {
        match self {
            Method::GET => http::Method::GET,
            Method::PUT => http::Method::PUT,
            Method::POST => http::Method::POST,
            Method::HEAD => http::Method::HEAD,
            Method::PATCH => http::Method::PATCH,
            Method::TRACE => http::Method::TRACE,
            Method::DELETE => http::Method::DELETE,
            Method::OPTIONS => http::Method::OPTIONS,
            Method::CONNECT => http::Method::CONNECT,
            _ => unreachable!(),
        }
    }
}
