use crate::endpoint::{Endpoint, WithContext};
use crate::http::{Method, Request, Response};
use crate::router::Router;
use crate::wrap::{Call, Wrap};
use crate::{Body, HasContext, Scope};

pub trait State: Send + Sync + Clone + 'static {}
impl<S> State for S where S: Send + Sync + Clone + 'static {}

pub struct Bison<W, S> {
    router: Router<W, S>,
    state: S,
}

impl Bison<Call, ()> {
    pub fn new() -> Bison<impl Wrap<()>, ()> {
        Self {
            router: Router::new(),
            state: (),
        }
    }
}

impl<S> Bison<Call, S>
where
    S: State,
{
    pub fn with_state(state: S) -> Bison<impl Wrap<S>, S> {
        Bison {
            router: Router::new(),
            state: state.into(),
        }
    }
}

impl<W, S> Bison<W, S>
where
    W: Wrap<S>,
    S: State,
{
    pub fn wrap<O>(self, wrap: O) -> Bison<impl Wrap<S>, S>
    where
        O: Wrap<S>,
    {
        Bison {
            router: self.router.wrap(wrap),
            state: self.state,
        }
    }

    pub fn route<E, P, C>(self, method: Method, path: P, endpoint: E) -> Self
    where
        P: Into<String>,
        E: Endpoint<C, S> + 'static,
        C: HasContext<S> + 'static,
    {
        Bison {
            state: self.state,
            router: self
                .router
                .route(method, path, WithContext::new(endpoint))
                .expect("failed to insert route"),
        }
    }

    pub async fn serve(&self, request: http::Request<Body>) -> Response {
        let request = Request {
            params: Vec::new(),
            inner: request,
            state: self.state.clone(),
        };

        self.router.serve(request).await
    }

    pub fn scope<M>(self, scope: Scope<M, S>) -> Self
    where
        M: Wrap<S> + Clone + 'static,
    {
        scope.register(self)
    }
}

macro_rules! insert_route {
    ($name:ident => Method::$method:ident) => {
        #[doc = concat!("Insert a route for the `", stringify!($method), "` method.")]
        pub fn $name<P, E, C>(self, path: P, endpoint: E) -> Self
        where
            P: Into<String>,
            E: Endpoint<C, S> + 'static,
            C: HasContext<S> + 'static,
        {
            self.route(Method::$method, path, endpoint)
        }
    };
}

impl<W, S> Bison<W, S>
where
    W: Wrap<S>,
    S: State,
{
    insert_route!(get => Method::GET);
    insert_route!(put => Method::PUT);
    insert_route!(post => Method::POST);
    insert_route!(delete => Method::DELETE);
    insert_route!(head => Method::HEAD);
    insert_route!(options => Method::OPTIONS);
    insert_route!(patch => Method::PATCH);
}
