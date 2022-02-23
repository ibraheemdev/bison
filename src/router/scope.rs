use crate::bounded::Rc;
use crate::handler::{BoxHandler, Handler};
use crate::http::Method;
use crate::wrap::{And, Call, Wrap};
use crate::Bison;

/// Routes scoped under a common prefix.
///
/// See [`Bison::scope`] for details.
pub struct Scope<W> {
    wrap: W,
    prefix: String,
    routes: Vec<(Method, String, BoxHandler)>,
}

impl Scope<Call> {
    pub(crate) fn new(prefix: impl Into<String>) -> Self {
        let mut prefix = prefix.into();

        if !prefix.starts_with('/') {
            prefix.insert(0, '/');
        }

        Self {
            wrap: Call,
            prefix: prefix.into(),
            routes: Vec::new(),
        }
    }
}

impl<W> Scope<W>
where
    W: Wrap,
{
    /// Insert a route for the given method.
    ///
    /// # Examples
    ///
    /// ```
    /// use bison::{Bison, Method};
    ///
    /// async fn home(req: &Request) -> Response {
    ///     Response::new("Hello world!")
    /// }
    ///
    /// let bison = Bison::new().scope("/api", |scope| {
    ///     scope
    ///         .route(Method::Get, "/users/:id", users::get)
    /// });
    /// # mod users { use bison::*; pub fn get(req: &Request) -> Response { todo!() } }
    /// ```
    pub fn route<H, S>(mut self, method: Method, path: &str, handler: H) -> Scope<W>
    where
        H: Handler<S> + 'static,
        S: Send + Sync + 'static,
    {
        // avoid registering "//foo"
        let path = if self.prefix == "/" && !path.is_empty() {
            path[1..].to_owned()
        } else {
            path.to_owned()
        };

        self.routes.push((method, path, handler.boxed()));
        self
    }

    /// Insert a route for the `GET` method.
    ///
    /// # Examples
    ///
    /// ```
    /// use bison::Bison;
    ///
    /// let bison = Bison::new().scope("/api", |scope| {
    ///     scope
    ///         .get("/users/:id", users::get)
    ///         .get("/posts/:id", posts::get)
    ///         .get("/admin/:token", admin::dashboard)
    /// });
    /// # use bison::*;
    /// # fn f(req: &Request) -> Response { todo!() }
    /// # mod users { pub use f as get; }
    /// # mod posts { pub use f as get; }
    /// # mod admin { pub use f as dashboard; }
    /// ```
    pub fn get<H, S>(self, path: &str, handler: H) -> Scope<W>
    where
        H: Handler<S> + 'static,
        S: Send + Sync + 'static,
    {
        self.route(Method::Get, path, handler)
    }

    /// Wrap the scope with some middleware.
    pub fn wrap<O>(self, wrap: O) -> Scope<And<W, O>>
    where
        O: Wrap,
    {
        Scope {
            wrap: self.wrap.wrap(wrap),
            prefix: self.prefix,
            routes: self.routes,
        }
    }

    route!(put => Put);
    route!(post => Post);
    route!(head => Head);
    route!(patch => Patch);
    route!(delete => Delete);
    route!(options => Options);

    pub(crate) fn register<O>(self, mut bison: Bison<O>) -> Bison<O>
    where
        O: Wrap,
    {
        let wrap = Rc::new(self.wrap);

        for (method, path, handler) in self.routes {
            bison = Bison {
                router: bison
                    .router
                    .route(
                        method,
                        format!("{}{}", self.prefix, path),
                        Box::new(handler.wrap(wrap.clone())),
                    )
                    .expect("failed to insert route"),
                state: bison.state,
            };
        }
        bison
    }
}

macro_rules! route {
    ($name:ident => $method:ident) => {
        #[doc = concat!("Insert a route for the `", stringify!($method), "` method.")]
        /// See [`get`](Scope::get) for examples.
        pub fn $name<H, S>(self, path: &str, handler: H) -> Scope<W>
        where
            H: Handler<S> + 'static,
            S: Send + Sync + 'static,
        {
            self.route(Method::$method, path, handler)
        }
    };
}

pub(self) use route;
