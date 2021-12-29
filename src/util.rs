use std::cell::UnsafeCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};

/// A thread-safe, mutable memory location with dynamically checked borrow rules.
pub struct AtomicRefCell<T> {
    borrowed: AtomicBool,
    value: UnsafeCell<T>,
}

impl<T> AtomicRefCell<T> {
    /// Create a new [`AtomicRefCell`] holding the given value.
    pub fn new(value: T) -> Self {
        Self {
            borrowed: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` exit scope.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        // synchronizes with the Release store in RefMut::drop
        if self.borrowed.swap(true, Ordering::Acquire) {
            panic!("already borrowed")
        }

        RefMut {
            value: self.value.get(),
            borrowed: &self.borrowed,
        }
    }
}

unsafe impl<T: Send> Send for AtomicRefCell<T> {}
unsafe impl<T: Send> Sync for AtomicRefCell<T> {}

/// A wrapper type for a mutably borrowed value from an [`AtomicRefCell`].
pub struct RefMut<'b, T> {
    value: *mut T,
    borrowed: &'b AtomicBool,
}

impl<'b, T> Drop for RefMut<'b, T> {
    fn drop(&mut self) {
        // synchronizes with the Acquire load in try_borrow
        self.borrowed.store(false, Ordering::Release)
    }
}

impl<'b, T> std::ops::Deref for RefMut<'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: we have exclusive access to
        // `value` through the `borrowed` flag
        unsafe { &*self.value }
    }
}

impl<'b, T> std::ops::DerefMut for RefMut<'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: we have exclusive access to
        // `value` through the `borrowed` flag
        unsafe { &mut *self.value }
    }
}

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
