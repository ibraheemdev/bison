use crate::bounded::{Send, Sync};
use crate::http::Response;
use crate::reject::IntoRejection;
use crate::wrap::{Next, Wrap};
use crate::Context;

use std::future::Future;
use std::marker::PhantomData;

/// Create middleware from a closure.
#[macro_export]
macro_rules! wrap_fn {
    (async |$req:ident, $next:ident| $body:expr) => {
        ::bison::wrap_fn!(async |$req: ::bison::Request, $next| $body)
    };
    (async |$req:ident: $cx:ty, $next:ident| $body:expr) => {{
        async fn wrap(
            $req: $cx,
            $next: &dyn ::bison::wrap::Next,
        ) -> Result<::bison::Response, ::bison::Rejection> {
            let response = $body;
            match response {
                Ok(r) => Ok(r),
                Err(e) => Err(::bison::Rejection::new(e)),
            }
        }

        ::bison::wrap::__internal_wrap_fn(wrap)
    }};
}

pub trait WrapFn<'a, C>: Send + Sync + 'static {
    type Error: IntoRejection;
    type Future: Future<Output = Result<Response, Self::Error>> + Send + 'a;

    fn call(&self, cx: C, next: &'a dyn Next) -> Self::Future;
}

impl<'a, F, C, O, E> WrapFn<'a, C> for F
where
    F: Fn(C, &'a dyn Next) -> O + Send + Sync + 'static,
    O: Future<Output = Result<Response, E>> + Send + 'a,
    E: IntoRejection,
{
    type Error = E;
    type Future = O;

    fn call(&self, cx: C, next: &'a dyn Next) -> Self::Future {
        self(cx, next)
    }
}

pub fn __internal_wrap_fn<'g, F, E, C>(f: F) -> impl Wrap<'g, C>
where
    for<'a> F: WrapFn<'a, C, Error = E>,
    E: IntoRejection + 'static,
    C: Context<'g>,
{
    struct Impl<F, E, C>(F, PhantomData<(E, C)>);

    #[crate::async_trait_internal]
    impl<'req, F, E, C> Wrap<'req, C> for Impl<F, E, C>
    where
        for<'a> F: WrapFn<'a, C, Error = E>,
        E: IntoRejection + 'static,
        C: Context<'req>,
    {
        type Rejection = E;

        async fn call(&self, cx: C, next: impl Next<'req>) -> Result<Response, Self::Rejection> {
            self.0.call(cx, next).await
        }
    }

    Impl(f, PhantomData::<(E, C)>)
}
