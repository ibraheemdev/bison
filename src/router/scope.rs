use crate::bounded::Rc;
use crate::handler::{self, Context, Erased, Handler};
use crate::http::Method;
use crate::wrap::{Call, Wrap};
use crate::{Bison, Request};

/// Routes scoped under a common prefix.
///
/// See [`Bison::scope`] for details.
pub struct Scope<W> {
    wrap: W,
    prefix: String,
    routes: Vec<(Method, String, Box<Erased>)>,
}

impl Scope<Call> {
    pub(crate) fn new(prefix: impl Into<String>) -> Self {
        let mut prefix = prefix.into();
        if !prefix.starts_with('/') {
            prefix.insert(0, '/');
        }

        Self {
            wrap: Call::new(),
            prefix: prefix.into(),
            routes: Vec::new(),
        }
    }
}

impl<W> Scope<W>
where
    W: Wrap<Request>,
{
    pub(crate) fn register<M>(self, mut bison: Bison<M>) -> Bison<impl Wrap>
    where
        M: Wrap<Request>,
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

    /// Wrap the routes with some middleware.
    pub fn wrap<O, C>(self, wrap: O) -> Scope<impl Wrap>
    where
        O: Wrap<C>,
        C: Context,
    {
        Scope {
            wrap: self.wrap.wrap(wrap),
            prefix: self.prefix,
            routes: self.routes,
        }
    }
}

macro_rules! route {
    ($name:ident => $method:ident) => {
        #[doc = concat!("Insert a route for the `", stringify!($method), "` method.")]
        pub fn $name<H, C>(mut self, path: &str, handler: H) -> Scope<impl Wrap>
        where
            H: Handler<C>,
            C: Context,
        {
            // avoid registering "//foo"
            let path = if self.prefix == "/" && !path.is_empty() {
                path[1..].to_owned()
            } else {
                path.to_owned()
            };

            self.routes
                .push((Method::$method, path, handler::erase(handler)));
            self
        }
    };
}

impl<W> Scope<W>
where
    W: Wrap<Request>,
{
    route!(get => GET);
    route!(put => PUT);
    route!(post => POST);
    route!(head => HEAD);
    route!(patch => PATCH);
    route!(delete => DELETE);
    route!(options => OPTIONS);
}
