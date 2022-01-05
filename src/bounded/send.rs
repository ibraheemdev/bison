use std::cell::UnsafeCell;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};

use futures_core::Stream;

pub use std::marker::{Send, Sync};

/// An dynamically typed [`Future`].
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// An dynamically typed [`Stream`].
pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + Sync + 'a>>;

/// An dynamically typed [`Error`].
pub type BoxError = Box<dyn Error + Send + Sync>;

pub use once_cell::sync::OnceCell;
pub use std::sync::Arc as Rc;

/// A macro for async-trait methods.
///
/// See [`async_trait`](https://docs.rs/async-trait/latest/async_trait/) for details.
pub use async_trait::async_trait;

/// `async-trait` but imports from `crate::`.
pub(crate) use async_trait::async_trait as async_trait_internal;

/// A thread-safe, mutable memory location with dynamically checked borrow rules.
#[derive(Default)]
pub struct RefCell<T> {
    borrowed: AtomicBool,
    value: UnsafeCell<T>,
}

impl<T> RefCell<T> {
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
impl<T: Clone> Clone for RefCell<T> {
    fn clone(&self) -> RefCell<T> {
        RefCell::new(self.borrow_mut().clone())
    }
}

impl<T: PartialEq> PartialEq for RefCell<T> {
    fn eq(&self, other: &RefCell<T>) -> bool {
        *self.borrow_mut() == *other.borrow_mut()
    }
}

unsafe impl<T: Send> Send for RefCell<T> {}
unsafe impl<T: Send> Sync for RefCell<T> {}

/// A wrapper type for a mutably borrowed value from a [`RefCell`].
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

// /// A thread-safe mutable memory location.
// pub struct Cell<T>(AtomicCell<T>);
// 
// impl<T> Cell<T> {
//     pub fn new(val: T) -> Self {
//         Self(AtomicCell::new(val))
//     }
// 
//     pub fn set(&self, val: T) {
//         self.0.store(val);
//     }
// }
// 
// impl<T: Copy> Cell<T> {
//     pub fn get(&self) -> T {
//         self.0.load()
//     }
// }
