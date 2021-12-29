use crate::bounded::{Send, Sync};
use crate::error::IntoResponseError;
use crate::http::{Request, Response};
use crate::wrap::{Next, Wrap};

use std::future::Future;
use std::marker::PhantomData;

/// Create middleware from a closure.
#[macro_export]
macro_rules! wrap_fn {
    (|$req:ident, $next:ident| $body:expr) => {{
        async fn wrap<'a>(
            $req: &'a ::bison::Request,
            $next: &'a dyn Next,
        ) -> Result<::bison::Response, ::bison::Error> {
            let response = $body;
            match response {
                Ok(r) => Ok(r),
                Err(e) => Err(::bison::Error::new(e)),
            }
        }

        ::bison::wrap::__internal_wrap_fn(wrap)
    }};
}

pub trait WrapFn<'a, E>: Send + Sync + 'static {
    type F: Future<Output = Result<Response, E>> + Send + 'a;

    fn call(&self, req: &'a Request, next: &'a dyn Next) -> Self::F;
}

impl<'a, O, E, F> WrapFn<'a, E> for F
where
    F: Fn(&'a Request, &'a dyn Next) -> O + Send + Sync + 'static,
    O: Future<Output = Result<Response, E>> + Send + 'a,
    E: IntoResponseError,
{
    type F = O;

    fn call(&self, req: &'a Request, next: &'a dyn Next) -> Self::F {
        self(req, next)
    }
}

pub fn __internal_wrap_fn<F, E>(f: F) -> impl Wrap
where
    for<'a> F: WrapFn<'a, E>,
    E: IntoResponseError + 'static,
{
    struct WrapFnImpl<F, E>(F, PhantomData<E>);

    #[crate::async_trait_internal]
    impl<F, E> Wrap for WrapFnImpl<F, E>
    where
        F: for<'a> WrapFn<'a, E>,
        E: IntoResponseError + 'static,
    {
        type Error = E;

        async fn call(&self, req: &Request, next: &impl Next) -> Result<Response, Self::Error> {
            self.0.call(req, &next).await
        }
    }

    WrapFnImpl(f, PhantomData::<E>)
}
