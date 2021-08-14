use std::future::Future;
use std::pin::Pin;

use futures_util::Stream;

#[cfg(feature = "not-send")]
mod not_send {
    use super::*;

    pub trait SendBound {}

    impl<T> SendBound for T {}

    pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
    pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + 'a>>;
    pub type BoxError = Box<dyn std::error::Error>;

    pub trait Boxed: Future + Sized {
        fn boxed<'a>(self) -> BoxFuture<'a, Self::Output>
        where
            Self: 'a,
        {
            Box::pin(self)
        }
    }

    impl<F> Boxed for F where F: Future {}
}

#[cfg(feature = "not-send")]
pub use not_send::*;

#[cfg(not(feature = "not-send"))]
mod send {
    use super::*;

    pub trait SendBound: Send + Sync {}

    impl<T> SendBound for T where T: Send + Sync {}

    pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'a>>;
    pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + Sync + 'a>>;
    pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

    pub trait Boxed: Future + Send + Sized + Sync {
        fn boxed<'a>(self) -> BoxFuture<'a, Self::Output>
        where
            Self: 'a,
        {
            Box::pin(self)
        }
    }

    impl<F> Boxed for F where F: Future + SendBound + 'static {}
}

#[cfg(not(feature = "not-send"))]
pub use send::*;
