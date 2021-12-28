use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct AtomicRefCell<T> {
    borrowed: AtomicBool,
    value: UnsafeCell<T>,
}

impl<T> AtomicRefCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            borrowed: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

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
