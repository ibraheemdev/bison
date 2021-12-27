//! This module is full of hacks around compiler limitations
//! with HRTBs :(

use crate::bounded::{Send, Sync};
use crate::{Handler, Responder, WithContext};

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{self, Poll};

impl<'a, F, O, R> Handler<'a, ()> for F
where
    F: Fn() -> O + Send + Sync,
    O: Future<Output = R> + Send + 'a,
    R: Responder,
{
    type Response = R;
    type Future = InfallibleFut<O>;
    type Error = Infallible;

    fn call(&'a self, _: ()) -> Self::Future {
        InfallibleFut { future: self() }
    }
}

impl<'a, F, C, O, R> Handler<'a, (C,)> for F
where
    F: FnArgs<C>,
    F: Fn(<C as WithContext<'a>>::Context) -> O + Send + Sync,
    O: Future<Output = R> + Send + 'a,
    R: Responder,
    C: WithContext<'a>,
{
    type Response = R;
    type Future = InfallibleFut<O>;
    type Error = Infallible;

    fn call(&self, req: C::Context) -> Self::Future {
        InfallibleFut { future: self(req) }
    }
}

pub trait FnArgs<A> {
    fn call(&self, args: A);
}

impl<F, O, A> FnArgs<A> for F
where
    F: Fn(A) -> O,
{
    fn call(&self, args: A) {
        self(args);
    }
}

pin_project_lite::pin_project! {
    pub struct InfallibleFut<F> {
        #[pin]
        future: F,
    }
}

impl<F: Future> Future for InfallibleFut<F> {
    type Output = Result<F::Output, Infallible>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.project().future.poll(cx).map(Ok)
    }
}
