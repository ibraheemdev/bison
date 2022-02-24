use crate::bounded::{BoxFuture, Send};
use crate::wrap::HandlerNext;
use crate::{Request, Result, State, Wrap};

use std::convert::Infallible;
use std::future::Future;
use std::marker::PhantomData;

/// An asynchronous HTTP handler.
#[async_trait::async_trait]
pub trait Handler<S = ()>: Send + Sync {
    /// Call this handler with the given request.
    async fn call(&self, req: Request) -> Result;

    /// Wrap this handler in some middleware.
    fn wrap<W>(self, wrap: W) -> Wrapped<Self, W, S>
    where
        Self: Sized,
        W: Wrap,
    {
        Wrapped {
            wrap,
            handler: HandlerNext(self, PhantomData),
            _state: PhantomData,
        }
    }

    /// Returns a type-erased handler.
    fn boxed(self) -> BoxHandler
    where
        Self: Sized + 'static,
        S: State,
    {
        struct Impl<H, S>(H, PhantomData<S>);

        impl<H, S> Handler for Impl<H, S>
        where
            H: Handler<S>,
            S: State,
        {
            fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result>
            where
                'a: 'o,
            {
                self.0.call(req)
            }
        }

        Box::new(Impl(self, PhantomData))
    }
}

/// A type-erased [`Handler`].
pub type BoxHandler = Box<dyn Handler>;

impl<S> Handler<S> for BoxHandler {
    fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result>
    where
        'a: 'o,
    {
        Handler::call(&**self, req)
    }
}

#[async_trait::async_trait]
impl<F, Fut> Handler<Infallible /* dummy type */> for F
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = Result> + Send + Sync,
{
    async fn call(&self, _req: Request) -> Result {
        self().await
    }
}

#[async_trait::async_trait]
impl<F, Fut> Handler<()> for F
where
    F: Fn(Request) -> Fut + Send + Sync,
    Fut: Future<Output = Result> + Send + Sync,
{
    async fn call(&self, req: Request) -> Result {
        self(req).await
    }
}

macro_rules! handler {
    ($( ( $($T:ident),* ), )*) => {$(
        #[async_trait::async_trait]
        #[allow(unused_parens, non_snake_case)]
        impl<Func, Fut, $($T),*> Handler<( $($T,)* )> for Func
        where
            Func: Fn(Request, $($T),*) -> Fut + Send + Sync,
            Fut: Future<Output = Result> + Send + Sync,
            $($T: State),*
        {
            async fn call(&self, req: Request) -> Result {
                let state = req.state.as_ref().unwrap();
                let ($($T),*) = ($(state.get::<$T>().cloned().unwrap()),*);
                self(req, $($T),*).await
            }
        }
    )*}
}

handler! {
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I),
    (A, B, C, D, E, F, G, H, I, J),
}

/// A handler wrapped in some middleware.
///
/// See [`Handler::wrap`] for details.
pub struct Wrapped<H, W, S> {
    handler: HandlerNext<H, S>,
    wrap: W,
    _state: PhantomData<S>,
}

impl<H, W, S> Handler<S> for Wrapped<H, W, S>
where
    H: Handler<S>,
    W: Wrap,
    S: State,
{
    fn call<'a, 'o>(&'a self, req: Request) -> BoxFuture<'o, Result>
    where
        'a: 'o,
    {
        self.wrap.call(req, &self.handler)
    }
}
