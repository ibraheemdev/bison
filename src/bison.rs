use crate::handler::Handler;
use crate::http::{Method, Request, Response};
use crate::router::{Router, Scope};
use crate::state::{self, State};
use crate::wrap::{And, Call, Wrap};

/// Where everything happens.
///
/// `Bison` is the entrypoint of your application. You can register HTTP
/// handlers, apply middleware, inject state, and register modules.
///
/// Most users will hand this off to a separate server crate, such as
/// [`bison_hyper`] or [`bison_actix`].
///
/// ```
/// use bison::Bison;
///
/// let bison = Bison::new()
///     .get("/home", home)
///     .get("/user/:id", get_user)
///     .wrap(Cors::all())
///     .register(Tera::new("./templates"))
///     .inject(Database::connect("localhost:20717"));
/// ```
pub struct Bison<W> {
    pub(crate) router: Router<W>,
    pub(crate) state: state::AppState,
}

impl Bison<Call> {
    /// Create a new `Bison`.
    ///
    /// ```
    /// use bison::Bison;
    ///
    /// let bison = Bison::new();
    /// ```
    pub fn new() -> Bison<Call> {
        Self {
            router: Router::new(),
            state: state::AppState::new(),
        }
    }
}

impl<W> Bison<W>
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
    /// let bison = Bison::new().route(Method::Get, "/", home);
    /// ```
    pub fn route<H>(self, method: Method, path: &str, handler: H) -> Bison<W>
    where
        H: Handler + 'static,
    {
        Bison {
            router: self
                .router
                .route(method, path, Box::new(handler))
                .expect("failed to insert route"),
            state: self.state,
        }
    }

    /// Insert a route for the `GET` method.
    ///
    /// # Examples
    ///
    /// ```
    /// use bison::Bison;
    ///
    /// async fn home() -> &'static str {
    ///     "Hello world!"
    /// }
    ///
    /// let bison = Bison::new().get("/", home);
    /// ```
    pub fn get<H, C>(self, path: &str, handler: H) -> Bison<W>
    where
        H: Handler + 'static,
    {
        self.route(Method::Get, path, handler)
    }

    route!(put => Put);
    route!(post => Post);
    route!(head => Head);
    route!(patch => Patch);
    route!(delete => Delete);
    route!(options => Options);

    /// Inject global application state.
    ///
    /// Any injected state will be accessible to handlers through the
    /// [`state`](crate::extract::state) extractor.
    ///
    /// ```
    /// # struct Database;
    /// # impl Database {
    /// #     fn connect(_: &str) -> Self { Self }
    /// #     async fn get_user(id: usize) -> String { String::new() }
    /// # }
    /// use bison::{Bison, Context};
    /// use bison::extract::state;
    ///
    /// #[derive(Context)]
    /// struct GetUser {
    ///     id: usize,
    ///     #[cx(state)]
    ///     db: &Database
    /// }
    ///
    /// async fn get_user(cx: GetUser) -> String {
    ///     let user = cx.db.get_user(cx.id).await;
    ///     format!("user: {}", user)
    /// }
    ///
    /// let database_url = std::env::var("DATABASE_URL").unwrap();
    /// let bison = Bison::new()
    ///     .get("/user/:id", get_user)
    ///     .inject(Database::connect(&database_url));
    /// ```
    pub fn inject<T>(self, state: T) -> Self
    where
        T: State,
    {
        Self {
            router: self.router,
            state: self
                .state
                .insert(state)
                .expect("cannot inject state after server has started"),
        }
    }

    /// Wrap the application with some middleware.
    pub fn wrap<O>(self, wrap: O) -> Bison<And<W, O>>
    where
        O: Wrap,
    {
        Bison {
            router: self.router.wrap(wrap),
            state: self.state,
        }
    }

    /// Register routes scoped under a common prefix.
    pub fn scope<F, O>(self, prefix: &str, f: F) -> Bison<W>
    where
        F: FnOnce(Scope<Call>) -> Scope<O>,
        O: Wrap,
    {
        let scope = f(Scope::new(prefix));
        scope.register(self)
    }

    /// Serve a single HTTP request.
    ///
    /// Most users will not interact with this method directly,
    /// and instead use a server crate such as [`bison_hyper`]
    /// or [`bison_actix`].
    pub async fn serve_one(&self, mut req: Request) -> Response {
        req.state = Some(self.state.clone());
        self.router.serve(req).await
    }
}

macro_rules! route {
    ($name:ident => $method:ident) => {
        #[doc = concat!("Insert a route for the `", stringify!($method), "` method.")]
        /// See [`get`](Bison::get) for examples.
        pub fn $name<H>(self, path: &str, handler: H) -> Bison<W>
        where
            H: Handler + 'static,
        {
            self.route(Method::$method, path, handler)
        }
    };
}

pub(self) use route;
