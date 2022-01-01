use crate::bounded::Rc;
use crate::handler::{self, Context, Erased, Handler};
use crate::http::Method;
use crate::wrap::{Call, Wrap};
use crate::Bison;

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
    W: Wrap,
{
    pub(crate) fn register<M>(self, mut bison: Bison<M>) -> Bison<impl Wrap>
    where
        M: Wrap,
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
    pub fn wrap<O>(self, wrap: O) -> Scope<impl Wrap>
    where
        O: Wrap,
    {
        Scope {
            wrap: self.wrap.and(wrap),
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
            self.routes
                .push((Method::$method, path.into(), handler::erase(handler)));
            self
        }
    };
}

impl<W> Scope<W>
where
    W: Wrap,
{
    route!(put => Put);
    route!(post => Post);
    route!(head => Head);
    route!(patch => Patch);
    route!(delete => Delete);
    route!(options => Options);
}
