use crate::endpoint::{Endpoint, WithContext};
use crate::http::{Method, Request, Response};
use crate::router::Router;
use crate::wrap::{Call, Wrap};
use crate::{HasContext, Scope, State};

pub struct Bison<W> {
    router: Router<W>,
    state: State,
}

impl Bison<Call> {
    pub fn new() -> Bison<impl Wrap> {
        Self {
            router: Router::new(),
            state: State::new(),
        }
    }
}

impl<W> Bison<W>
where
    W: Wrap,
{
    pub fn wrap<O>(self, wrap: O) -> Bison<impl Wrap>
    where
        O: Wrap,
    {
        Bison {
            router: self.router.wrap(wrap),
            state: self.state,
        }
    }

    pub fn route<E, P, C>(self, method: Method, path: P, endpoint: E) -> Self
    where
        P: Into<String>,
        E: Endpoint<C> + 'static,
        C: HasContext + 'static,
    {
        Bison {
            state: self.state,
            router: self
                .router
                .route(method, path, WithContext::new(endpoint))
                .expect("failed to insert route"),
        }
    }

    pub fn state<T>(mut self, state: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.state.insert(state);
        self
    }

    pub async fn serve(&self, mut request: Request) -> Response {
        request.extensions_mut().insert::<State>(self.state.clone());
        self.router.serve(request).await
    }

    pub fn scope<S>(self, scope: Scope<S>) -> Self
    where
        S: Wrap + Clone + 'static,
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
            E: Endpoint<C> + 'static,
            C: HasContext + 'static,
        {
            self.route(Method::$method, path, endpoint)
        }
    };
}

impl<W> Bison<W>
where
    W: Wrap,
{
    insert_route!(get => Method::GET);
    insert_route!(put => Method::PUT);
    insert_route!(post => Method::POST);
    insert_route!(delete => Method::DELETE);
    insert_route!(head => Method::HEAD);
    insert_route!(options => Method::OPTIONS);
    insert_route!(patch => Method::PATCH);
}
