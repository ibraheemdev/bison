use crate::handler::{self, Context, Handler};
use crate::http::{Body, Method, Response};
use crate::router::{Router, Scope};
use crate::state::{self, State};
use crate::wrap::{Call, Wrap};
use crate::Request;

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
pub struct Bison<'g, W> {
    pub(crate) router: Router<W>,
    pub(crate) state: state::AppState,
    guard: &'g (),
}

impl<'g> Bison<'g, Call> {
    /// Create a new `Bison`.
    ///
    /// ```
    /// use bison::Bison;
    ///
    /// let bison = Bison::new();
    /// ```
    pub unsafe fn new(guard: &'g ()) -> Bison<'g, impl Wrap<&'g Request>> {
        Self {
            router: Router::new(),
            state: state::AppState::new(),
            guard,
        }
    }
}

impl<'g, W> Bison<'g, W>
where
    W: Wrap<'g, &'g Request>,
{
    /// Insert a route for the given method.
    ///
    /// # Examples
    ///
    /// ```
    /// use bison::{Bison, Method};
    ///
    /// async fn home() -> &'static str {
    ///     "Hello world!"
    /// }
    ///
    /// let bison = Bison::new().get("/", Method::Get, home);
    /// ```
    pub fn route<H, C>(
        self,
        path: &'g str, // TODO?
        method: Method,
        handler: H,
    ) -> Bison<'g, impl Wrap<&'g Request>>
    where
        H: Handler<C> + 'static,
        C: Context<'g>,
    {
        // SAFETY: 'g acting as an HRTB is guaranteed by Scope::new
        let handler = unsafe { handler::erase(handler, self.guard) };

        Bison {
            router: self
                .router
                .route(method, path, handler)
                .expect("failed to insert route"),
            state: self.state,
            guard: self.guard,
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
    pub fn get<H, C>(self, path: &'g str, handler: H) -> Bison<impl Wrap<&'g Request>>
    where
        H: Handler<C> + 'static,
        C: Context<'g>,
    {
        self.route(path, Method::GET, handler)
    }

    route!(put => PUT);
    route!(post => POST);
    route!(head => HEAD);
    route!(patch => PATCH);
    route!(delete => DELETE);
    route!(options => OPTIONS);

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
            guard: self.guard,
        }
    }

    /// Wrap the application with some middleware.
    pub fn wrap<O, C>(self, wrap: O) -> Bison<'g, impl Wrap<'g, &'g Request>>
    where
        O: Wrap<'g, C>,
        C: Context<'g>,
    {
        Bison {
            router: self.router.wrap(wrap),
            state: self.state,
            guard: self.guard,
        }
    }

    /// Register routes scoped under a common prefix.
    pub fn scope<F, O>(self, prefix: &'g str, f: F) -> Bison<'g, impl Wrap<&'g Request>>
    where
        F: FnOnce(Scope<'g, Call>) -> Scope<'g, O>,
        O: Wrap<'g, &'g Request>,
    {
        let scope = unsafe { Scope::new(prefix, self.guard) };
        let scope = f(scope);
        scope.register(self)
    }

    /// Serve a single HTTP request.
    ///
    /// Most users will not interact with this method directly,
    /// and instead use a server crate such as [`bison_hyper`]
    /// or [`bison_actix`].
    pub async fn serve_one(&self, req: http::Request<Body>) -> Response {
        self.router.serve(req, self.state.clone()).await
    }
}

macro_rules! route {
    ($name:ident => $method:ident) => {
        #[doc = concat!("Insert a route for the `", stringify!($method), "` method.")]
        /// See [`get`](Self::get) for examples.
        pub fn $name<H, C>(self, path: &'g str, handler: H) -> Bison<'g, impl Wrap<&'g Request>>
        where
            H: Handler<C> + 'static,
            C: Context<'g>,
        {
            self.route(path, Method::$method, handler)
        }
    };
}

pub(self) use route;
