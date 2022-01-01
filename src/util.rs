use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub async fn poll_fn<T, F>(f: F) -> T
where
    F: FnMut(&mut Context<'_>) -> Poll<T>,
{
    struct PollFn<F>(F);

    impl<F> Unpin for PollFn<F> {}

    impl<T, F> Future for PollFn<F>
    where
        F: FnMut(&mut Context<'_>) -> Poll<T>,
    {
        type Output = T;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
            (self.0)(cx)
        }
    }

    PollFn(f).await
}

macro_rules! cfg_json {
    ($($x:item)*) => {$(
        #[cfg(feature = "json")]
        $x
    )*}
}

macro_rules! doc_inline {
    ($($x:item)*) => {$(
        #[doc(inline)]
        $x
    )*}
}

pub(crate) use {cfg_json, doc_inline};

macro_rules! _try {
    ($expr:expr) => {{
        (|| $expr)()
    }};
}

pub(crate) use _try;
