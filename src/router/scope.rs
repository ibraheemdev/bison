use crate::bounded::Rc;
use crate::handler::{self, Context, Erased, Handler, Static};
use crate::http::Method;
use crate::wrap::{Call, Wrap};
use crate::{Bison, Request};

/// Routes scoped under a common prefix.
///
/// See [`Bison::scope`] for details.
pub struct Scope<'g, W> {
    wrap: W,
    prefix: String,
    routes: Vec<(Method, String, Erased)>,
    guard: &'g (),
}

impl<'g> Scope<'g, Call> {
    pub(crate) unsafe fn new(prefix: impl Into<String>, guard: &'g ()) -> Self {
        let mut prefix = prefix.into();
        if !prefix.starts_with('/') {
            prefix.insert(0, '/');
        }

        Self {
            wrap: Call::new(),
            prefix: prefix.into(),
            routes: Vec::new(),
            guard,
        }
    }
}

impl<'g, W> Scope<'g, W>
where
    W: Wrap<'g, &'g Request> + Static,
{
    pub(crate) fn register<M>(self, mut bison: Bison<'g, M>) -> Bison<'g, impl Wrap<&'g Request>>
    where
        M: Wrap<'g, &'g Request>,
    {
        let wrap = Rc::new(self.wrap);
        for (method, path, handler) in self.routes {
            bison = Bison {
                router: bison
                    .router
                    .route(method, format!("{}{}", self.prefix, path), unsafe {
                        handler::erase(handler.wrap(wrap.clone()), self.guard)
                    })
                    .expect("failed to insert route"),
                state: bison.state,
                guard: self.guard,
            };
        }
        bison
    }

    /// Wrap the routes with some middleware.
    pub fn wrap<O, C>(self, wrap: O) -> Scope<'g, impl Wrap<'g, &'g Request>>
    where
        O: Wrap<'g, C>,
        C: Context<'g>,
    {
        Scope {
            wrap: self.wrap.wrap(wrap),
            prefix: self.prefix,
            routes: self.routes,
            guard: self.guard,
        }
    }

    pub fn route<H, C>(
        self,
        path: &str,
        method: Method,
        handler: H,
    ) -> Scope<'g, impl Wrap<&'g Request>>
    where
        H: Handler<C> + 'static,
        C: Context<'g>,
    {
        // avoid registering "//foo"
        let path = if self.prefix == "/" && !path.is_empty() {
            path[1..].to_owned()
        } else {
            path.to_owned()
        };

        // SAFETY: 'g acting as an HRTB is guaranteed by Scope::new
        let handler = unsafe { handler::erase(handler, self.guard) };
        self.routes.push((method, path, handler));
        self
    }
}

macro_rules! route {
    ($name:ident => $method:ident) => {
        #[doc = concat!("Insert a route for the `", stringify!($method), "` method.")]
        pub fn $name<H, C>(mut self, path: &str, handler: H) -> Scope<'g, impl Wrap<&'g Request>>
        where
            H: Handler<C> + 'static,
            C: Context<'g>,
        {
            self.route(path, Method::$method, handler)
        }
    };
}

impl<'g, W> Scope<'g, W>
where
    W: Wrap<'g, Request>,
{
    route!(get => GET);
    route!(put => PUT);
    route!(post => POST);
    route!(head => HEAD);
    route!(patch => PATCH);
    route!(delete => DELETE);
    route!(options => OPTIONS);
}
