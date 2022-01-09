use futures_core::Stream;
use std::cell::UnsafeCell;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;

pub use once_cell::unsync::OnceCell;
pub use std::cell::{Cell, RefCell};
pub use std::rc::Rc;

pub trait Send {}

impl<T> Send for T {}

pub trait Sync {}

impl<T> Sync for T {}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + 'a>>;
pub type BoxError = Box<dyn Error>;

pub use bison_codegen::async_trait_not_send as async_trait;
pub(crate) use bison_codegen::async_trait_not_send_internal as async_trait_internal;

pub struct LocalCell<T> {
    value: UnsafeCell<T>,
    #[cfg(debug_assertions)]
    borrowed: std::cell::Cell<bool>,
    _not_send: *mut (),
}

impl<T> LocalCell<T> {
    pub fn new(value: T) -> LocalCell<T> {
        LocalCell {
            value: UnsafeCell::new(value),
            #[cfg(debug_assertions)]
            borrowed: Default::default(),
            _not_send: std::ptr::null_mut(),
        }
    }

    /// # Safety
    ///
    /// `with` must not be called again within the closure.
    pub unsafe fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        #[cfg(debug_assertions)]
        if self.borrowed.replace(true) {
            panic!("attempted to borrow LocalCell twice");
        }

        // SAFETY:
        // - caller guarantees that `with` will
        //  not be called in `f`, and that is the only
        //  way to get a reference to `val`.
        // - LocalCell is !Sync
        let val = unsafe { &mut *self.value.get() };
        let val = f(val);

        #[cfg(debug_assertions)]
        self.borrowed.set(false);

        val
    }
}
