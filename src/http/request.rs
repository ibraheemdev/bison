use super::Body;
use crate::bounded::{cfg_send, Cell, OnceCell, Rc, RefCell};
use crate::state::{AppState, State};

use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::str::FromStr;
use std::sync::atomic::{AtomicPtr, Ordering};

/// An HTTP method.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Method {
    Get,
    Put,
    Post,
    Delete,
    Options,
    Head,
    Trace,
    Connect,
    Patch,
}

/// An HTTP request.
#[derive(Clone)]
pub struct Request {
    shared: Rc<Shared>,
}

pub struct Shared {
    method: Cell<Method>,
    uri: UriCell,
    state: AppState,
    headers: Headers,
    cache: Cache,
    body: Body,
    route_params: Params,
    query_params: OnceCell<Params>,
}

impl Request {
    pub fn method(&self) -> Method {
        self.shared.method.get()
    }

    pub fn set_method(&self, method: Method) {
        self.shared.method.set(method);
    }

    pub fn uri(&self) -> &Uri {
        self.shared.uri.get()
    }

    pub fn set_uri(&self, uri: Uri) {
        self.shared.uri.set(uri);
    }

    pub fn headers(&self) -> &Headers {
        &self.shared.headers
    }

    pub fn param(&self, name: &str) -> Option<&str> {
        self.shared.route_params.get(name)
    }

    pub fn query(&self, name: &str) -> Option<&str> {
        if let Some(query) = self.shared.uri.get().query() {
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

const INVALID_METHOD: &str = "invalid HTTP method";

impl Request {
    pub(crate) fn new(req: http::Request<Body>, state: AppState, route_params: Params) -> Self {
        let (req, body) = req.into_parts();

        Request {
            shared: Rc::new(Shared {
                method: Cell::new(Method::from_http(req.method)),
                uri: UriCell::new(Uri(req.uri)),
                query_params: OnceCell::new(),
                headers: Headers(RefCell::new(req.headers)),
                cache: Cache::default(),
                route_params,
                body,
                state,
            }),
        }
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

#[derive(Clone, PartialEq, Default)]
pub struct Headers(RefCell<http::HeaderMap>);

impl Headers {
    pub fn get(&self, key: http::header::HeaderName) -> Option<http::HeaderValue> {
        self.0.borrow_mut().get(key).cloned()
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
    fn from_http(method: http::Method) -> Self {
        match method.as_str().len() {
            3 => match method {
                http::Method::GET => Method::Get,
                http::Method::PUT => Method::Put,
                _ => panic!("{}", INVALID_METHOD),
            },
            4 => match method {
                http::Method::POST => Method::Post,
                http::Method::HEAD => Method::Head,
                _ => panic!("{}", INVALID_METHOD),
            },
            5 => match method {
                http::Method::PATCH => Method::Patch,
                http::Method::TRACE => Method::Trace,
                _ => panic!("{}", INVALID_METHOD),
            },
            6 => match method {
                http::Method::DELETE => Method::Delete,
                _ => panic!("{}", INVALID_METHOD),
            },
            7 => match method {
                http::Method::OPTIONS => Method::Options,
                http::Method::CONNECT => Method::Connect,
                _ => panic!("{}", INVALID_METHOD),
            },
            _ => panic!("{}", INVALID_METHOD),
        }
    }

    pub(crate) fn into_http(self) -> http::Method {
        match self {
            Method::Get => http::Method::GET,
            Method::Put => http::Method::PUT,
            Method::Post => http::Method::POST,
            Method::Head => http::Method::HEAD,
            Method::Patch => http::Method::PATCH,
            Method::Trace => http::Method::TRACE,
            Method::Delete => http::Method::DELETE,
            Method::Options => http::Method::OPTIONS,
            Method::Connect => http::Method::CONNECT,
        }
    }
}

/// A linked-list of `Uri`s.
///
/// URIs are likely to be read a lot, and only maybe
/// mutated conditionally in a middleware. By never
/// deallocating new URI values until the request is
/// dropped, we impose an extra allocation for
/// stores but make reads effectively free.
struct UriCell {
    root: AtomicPtr<UriNode>,
}

struct UriNode {
    uri: Uri,
    next: AtomicPtr<UriNode>,
}

impl UriCell {
    pub fn new(uri: Uri) -> Self {
        Self {
            root: AtomicPtr::new(Box::into_raw(Box::new(UriNode {
                uri,
                next: AtomicPtr::new(std::ptr::null_mut()),
            }))),
        }
    }

    pub fn get(&self) -> &Uri {
        // SAFETY: nodes are never deallocated until the list is
        // dropped, and root is never null
        unsafe { &(*self.root.load(Ordering::Acquire)).uri }
    }

    pub fn set(&self, uri: Uri) {
        let node = Box::into_raw(Box::new(UriNode {
            uri,
            // Technically we could lose a node in between this
            // load and the store, but the request is going to be
            // handled by one task anyways.
            next: AtomicPtr::new(self.root.load(Ordering::Acquire)),
        }));

        self.root.store(node, Ordering::Release);
    }
}

impl Drop for UriCell {
    fn drop(&mut self) {
        let mut node = *self.root.get_mut();
        while !node.is_null() {
            // SAFETY: &mut self guarantees we have unique
            // access to the nodes, which were create from
            // Box::into_raw
            let mut uri = unsafe { Box::from_raw(node) };
            node = *uri.next.get_mut();
        }
    }
}

#[test]
fn uricell() {
    let uri = UriCell::new("https://www.rust-lang.org/install.html".parse().unwrap());
    assert_eq!(
        uri.get().to_string(),
        "https://www.rust-lang.org/install.html"
    );
    uri.set("www.golang.org".parse().unwrap());
    assert_eq!(uri.get().to_string(), "www.golang.org");
    uri.set("www.goolang.org".parse().unwrap());
    assert_eq!(uri.get().to_string(), "www.goolang.org");
    drop(uri);
}
