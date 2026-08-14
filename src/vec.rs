use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::copy_nonoverlapping,
};

use crate::boxed::Box;

pub struct Vec<T> {
    /// # Safety
    ///
    /// The subslice `..len` of this buffer must only contain initialized values
    /// of `T`.
    buf: Box<[MaybeUninit<T>]>,
    /// # Safety
    ///
    /// `len` must be less than or equal to `buf.len()`.
    len: usize,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        // SAFETY: Since `len` is zero, no elements need to be initialized.
        Self {
            buf: Box::new_uninit_slice(0),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        // SAFETY: Since `len` is zero, no elements need to be initialized.
        Self {
            buf: Box::new_uninit_slice(capacity),
            len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn push(&mut self, value: T) {
        let value = MaybeUninit::new(value);

        if self.buf.len() == self.len {
            let mut new_buf = Box::new_uninit_slice(self.len + 1);

            // SAFETY: Both pointers are properly aligned and are valid for
            // reads/writes respectively for `len` and `len + 1` values. Since
            // these are both separate allocations, the pointers do not overlap.
            unsafe {
                copy_nonoverlapping::<MaybeUninit<T>>(
                    self.buf.as_ptr(),
                    new_buf.as_mut_ptr(),
                    self.len,
                )
            };

            self.buf = new_buf;
        }

        // SAFETY: `self.len` is guaranteed to be in bounds since we checked
        // that `cap > len`.
        let slot = unsafe { self.buf.get_unchecked_mut(self.len) };
        *slot = value;

        // SAFETY: `self.buf.len()` cannot go over `isize::MAX`, and
        // `len < cap` is checked. This also guarantees the new `len` is
        // still in bounds of `buf`.
        self.len = unsafe { self.len.unchecked_add(1) };
    }

    pub fn pop(&mut self) -> Option<T> {
        // If the vector is empty, this line returns early.
        // SAFETY: `len <= cap` is guaranteed to be kept here. If `len` was
        // already zero, it stays zero and the condition is met.
        self.len = self.len.checked_sub(1)?;

        // SAFETY: The index is in bounds since `len < cap`, since we just
        // decremented `len`.
        let slot = unsafe { self.buf.get_unchecked_mut(self.len) };

        // SAFETY: This slot contains an initialized value, since it was an in
        // bounds element before we decremented `len`. This slot will not be
        // assumed init again, since it is now out of `len` bounds.
        Some(unsafe { slot.assume_init_read() })
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // SAFETY: The subslice `..len` of the buffer is guaranteed to only
        // contain initialized values of `T`
        unsafe { self.buf[..self.len].assume_init_ref() }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The subslice `..len` of the buffer is guaranteed to only
        // contain initialized values of `T`
        unsafe { self.buf[..self.len].assume_init_mut() }
    }
}

#[cfg(test)]
mod tests {
    use crate::vec::Vec;

    const _: () = {
        assert!(size_of::<Vec<i32>>() == size_of::<usize>() * 3);
        assert!(size_of::<Option<Vec<i32>>>() == size_of::<usize>() * 3);
        assert!(size_of::<Vec<()>>() == size_of::<usize>() * 3);
        assert!(size_of::<Option<Vec<()>>>() == size_of::<usize>() * 3);
    };

    #[test]
    fn test_new() {
        let vec = Vec::<i32>::new();
        assert!(vec.is_empty());
        assert_eq!(vec.capacity(), 0);
    }

    #[test]
    fn test_with_capacity() {
        let vec = Vec::<i32>::with_capacity(5);
        assert!(vec.is_empty());
        assert_eq!(vec.capacity(), 5);
    }

    #[test]
    fn test_push() {
        let mut vec = Vec::new();
        vec.push(4);
        vec.push(3);
        vec.push(5);
        assert_eq!(&*vec, &[4, 3, 5]);
        assert_eq!(vec.capacity(), 3);

        let mut vec = Vec::with_capacity(5);
        vec.push(4);
        vec.push(3);
        vec.push(5);
        assert_eq!(&*vec, &[4, 3, 5]);
        assert_eq!(vec.capacity(), 5);
    }

    #[test]
    fn test_pop() {
        let mut vec = Vec::new();
        vec.push(4);
        vec.push(3);
        vec.push(5);
        assert_eq!(vec.pop(), Some(5));
        assert_eq!(&*vec, &[4, 3]);
        assert_eq!(vec.capacity(), 3);

        let mut vec = Vec::with_capacity(5);
        vec.push(4);
        vec.push(3);
        vec.push(5);
        assert_eq!(vec.pop(), Some(5));
        assert_eq!(&*vec, &[4, 3]);
        assert_eq!(vec.capacity(), 5);

        let mut vec = Vec::<i32>::new();
        assert_eq!(vec.pop(), None);
        assert!(vec.is_empty());
        assert_eq!(vec.capacity(), 0);

        let mut vec = Vec::<i32>::with_capacity(5);
        assert_eq!(vec.pop(), None);
        assert!(vec.is_empty());
        assert_eq!(vec.capacity(), 5);
    }
}
