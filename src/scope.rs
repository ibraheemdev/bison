use crate::bison::State;
use crate::endpoint::{Endpoint, WithContext};
use crate::http::{Method, Request};
use crate::send::BoxFuture;
use crate::wrap::{Call, Next, Wrap};
use crate::{Bison, Error, HasContext, Response};

pub struct Scope<W, S>
where
    S: State,
{
    wrap: W,
    prefix: String,
    routes: Vec<(
        Method,
        String,
        Box<dyn Endpoint<Request<S>, S, Error = Error>>,
    )>,
}

impl<S> Scope<Call, S>
where
    S: State,
{
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

impl<W, S> Scope<W, S>
where
    S: State,
    W: Wrap<S> + Clone + 'static,
{
    pub(crate) fn register<M>(self, mut bison: Bison<M, S>) -> Bison<M, S>
    where
        M: Wrap<S>,
    {
        for (method, path, endpoint) in self.routes {
            bison = bison.route(
                method,
                format!("{}{}", self.prefix, path),
                endpoint.wrap(self.wrap.clone()),
            );
        }
        bison
    }

    pub fn wrap<O>(self, wrap: O) -> Scope<impl Wrap<S>, S>
    where
        O: Wrap<S>,
    {
        Scope {
            wrap: self.wrap.and(wrap),
            prefix: self.prefix,
            routes: self.routes,
        }
    }
}

macro_rules! insert_route {
    ($name:ident => Method::$method:ident) => {
        #[doc = concat!("Insert a route for the `", stringify!($method), "` method.")]
        pub fn $name<P, E, C>(&mut self, path: P, endpoint: E) -> &mut Self
        where
            P: Into<String>,
            E: Endpoint<C, S> + 'static,
            C: HasContext<S> + 'static,
        {
            self.routes.push((
                Method::$method,
                path.into(),
                Box::new(WithContext::new(endpoint)),
            ));
            self
        }
    };
}

impl<W, S> Scope<W, S>
where
    S: State,
    W: Wrap<S> + Clone + 'static,
{
    insert_route!(get => Method::GET);
    insert_route!(put => Method::PUT);
    insert_route!(post => Method::POST);
    insert_route!(delete => Method::DELETE);
    insert_route!(head => Method::HEAD);
    insert_route!(options => Method::OPTIONS);
}

struct ScopedWrap<W> {
    wrap: W,
    prefix: String,
}

impl<S, W> Wrap<S> for ScopedWrap<W>
where
    S: State,
    W: Wrap<S>,
{
    type Error = Error;

    fn wrap<'a>(
        &'a self,
        req: Request<S>,
        next: impl Next<'a, S>,
    ) -> BoxFuture<'a, Result<Response, Self::Error>> {
        if req.uri().path().starts_with(&self.prefix) {
            Box::pin(async move { self.wrap.wrap(req, next).await.map_err(Into::into) })
        } else {
            Box::pin(next.serve(req))
        }
    }
}
