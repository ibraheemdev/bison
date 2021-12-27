use crate::bounded::Rc;
use crate::handler::{self, Erased, Handler, HandlerExt};
use crate::http::Method;
use crate::wrap::{And, Call, Wrap};
use crate::{Bison, WithContext};

pub struct Scope<W> {
    wrap: W,
    prefix: String,
    routes: Vec<(Method, String, Box<Erased>)>,
}

impl Scope<Call> {
    pub fn new(prefix: impl Into<String>) -> Self {
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
            bison = bison.route(
                &format!("{}{}", self.prefix, path),
                method,
                handler.wrap(wrap.clone()),
            );
        }
        bison
    }

    pub fn wrap<O>(self, wrap: O) -> Scope<impl Wrap>
    where
        O: Wrap,
    {
        Scope {
            wrap: And {
                inner: self.wrap,
                outer: wrap,
            },
            prefix: self.prefix,
            routes: self.routes,
        }
    }
}

macro_rules! insert_route {
    ($name:ident => Method::$method:ident) => {
        #[doc = concat!("Insert a route for the `", stringify!($method), "` method.")]
        pub fn $name<H, C>(mut self, path: &str, handler: H) -> Scope<impl Wrap>
        where
            H: for<'r> Handler<'r, C> + 'static,
            C: for<'r> WithContext<'r> + 'static,
        {
            self.routes.push((
                Method::$method,
                path.into(),
                Box::new(handler::BoxReturn::new(handler::Extract::new(handler))),
            ));
            self
        }
    };
}

impl<W> Scope<W>
where
    W: Wrap,
{
    insert_route!(get => Method::GET);
    insert_route!(put => Method::PUT);
    insert_route!(post => Method::POST);
    insert_route!(delete => Method::DELETE);
    insert_route!(head => Method::HEAD);
    insert_route!(options => Method::OPTIONS);
}
