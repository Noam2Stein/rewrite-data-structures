use core::{cell::Cell, mem::MaybeUninit, ops::Deref, ptr::NonNull};

use crate::boxed::Box;

pub struct Rc<T: ?Sized> {
    /// # Safety
    ///
    /// This pointer must initially come from `Box::into_raw`. It must not be
    /// deallocated until `strong_count` and `weak_count` are both zero, which
    /// cannot happen until `self` is dropped. Mutable references must not be
    /// created from this pointer, as shared references can always exist.
    ///
    /// This pointer must point to a fully initialized value of `RcInner<T>`.
    ///
    /// Incorrectly changing `strong_count` and `weak_count` could result in
    /// undefined behavior.
    ptr: NonNull<RcInner<T>>,
}

pub struct Weak<T: ?Sized> {
    /// # Safety
    ///
    /// This pointer must initially come from `Box::into_raw`. It must not be
    /// deallocated until `strong_count` and `weak_count` are both zero. Mutable
    /// references must not be created from this pointer, as shared references
    /// can always exist.
    ///
    /// If `strong_count` is zero, `value` may not be initialized. In this case
    /// the pointer cannot be fully dereferenced, so to access `weak_count` the
    /// offset and cast must occur before converting to a reference.
    ///
    /// Incorrectly changing `strong_count` and `weak_count` could result in
    /// undefined behavior.
    ptr: NonNull<RcInner<T>>,
}

#[repr(C)]
struct RcInner<T: ?Sized> {
    /// The number of strong pointers.
    strong_count: Cell<usize>,
    /// The number of strong and weak pointers.
    weak_count: Cell<usize>,
    value: T,
}

impl<T: ?Sized> Rc<T> {
    pub fn new(value: T) -> Rc<T>
    where
        T: Sized,
    {
        // SAFETY: The pointer does come from `Box::into_raw`. Both counts are
        // correct, as this is the only RC at this point.
        Self {
            ptr: Box::into_raw(Box::new(RcInner {
                strong_count: Cell::new(1),
                weak_count: Cell::new(1),
                value,
            })),
        }
    }
}

impl<T: ?Sized> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Since this RC exists, `strong_count` must be greater than
        // zero, so the entire `RcInner` must be initialized. Returning the
        // shared reference is sound because it will not be deallocated until
        // `self` is dropped, and mutable references are not allowed to be
        // created.
        unsafe { &self.ptr.as_ref().value }
    }
}

impl<T: ?Sized> Drop for Rc<T> {
    fn drop(&mut self) {
        let boxed = self.ptr.assume_init_ref();
    }
}
