use crate::context::WithContext;
use crate::handler::Handler;
use crate::http::{Method, Request, Response};
use crate::router::Router;
use crate::state;
use crate::wrap::{Call, Wrap};

pub struct Bison<W> {
    router: Router<W>,
    state: state::Map,
}

impl Bison<Call> {
    pub fn new() -> Bison<impl Wrap<'static>> {
        Self {
            router: Router::new(),
            state: state::Map::new(),
        }
    }
}

impl<W> Bison<W>
where
    W: for<'req> Wrap<'req>,
{
    pub async fn serve(&self, mut req: Request) -> Response {
        req.extensions_mut().insert(self.state.clone());
        self.router.serve(req).await
    }
}

macro_rules! insert_route {
    ($name:ident => Method::$method:ident) => {
        #[doc = concat!("Insert a route for the `", stringify!($method), "` method.")]
        /// ```rust
        /// async fn home(_: ()) -> &'static str {
        ///     "Welcome!"
        /// }
        ///
        /// let bison = Bison::new();
        /// bison.get("/home", home);
        /// ```
        pub fn $name<H, C>(self, path: &str, handler: H) -> Bison<impl for<'req> Wrap<'req>>
        where
            H: for<'req> Handler<'req, C>,
            C: for<'req> WithContext<'req>,
        {
            let router = self
                .router
                .route(Method::$method, path, handler)
                .expect("failed to insert route");

            Bison {
                router,
                state: self.state,
            }
        }
    };
}

impl<W> Bison<W>
where
    W: for<'req> Wrap<'req>,
{
    insert_route!(get => Method::GET);
    insert_route!(put => Method::PUT);
    insert_route!(post => Method::POST);
    insert_route!(delete => Method::DELETE);
    insert_route!(head => Method::HEAD);
    insert_route!(options => Method::OPTIONS);
    insert_route!(patch => Method::PATCH);
}
