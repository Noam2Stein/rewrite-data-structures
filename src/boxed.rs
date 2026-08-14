use alloc::alloc::{alloc, handle_alloc_error};
use core::{
    alloc::Layout,
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr::{NonNull, drop_in_place, read},
};

pub struct Box<T: ?Sized>(
    /// # Safety
    ///
    /// This pointer must be convertable to a reference.
    NonNull<T>,
);

impl<T> Box<T> {
    pub fn new(value: T) -> Self {
        let value = ManuallyDrop::new(value);

        if size_of::<T>() == 0 {
            Self(NonNull::dangling())
        } else {
            let layout = Layout::new::<T>();

            // SAFETY: `layout` has a non-zero size
            let ptr = unsafe { alloc(layout) };

            let Some(ptr) = NonNull::new(ptr.cast::<T>()) else {
                handle_alloc_error(layout);
            };

            // SAFETY: `ptr` is properly aligned and points to a valid
            // allocation we just made.
            unsafe { ptr.write(ManuallyDrop::into_inner(value)) };

            // SAFETY: `ptr` points to a properly initialized value of `T`,
            // which will not be dropped until `self` is dropped. The pointer
            // is also properly aligned.
            Self(ptr)
        }
    }

    pub fn into_inner(boxed: Box<T>) -> T {
        let boxed = ManuallyDrop::new(boxed);

        // The pointer is guaranteed to be aligned, and we have exclusive access
        // to the pointed at value.
        unsafe { read(boxed.0.as_ptr()) }
    }
}

impl<T> Box<[T]> {
    pub fn new_uninit_slice(len: usize) -> Box<[MaybeUninit<T>]> {
        if size_of::<T>() == 0 || len == 0 {
            // SAFETY: If `T` is a ZST, then a dangling pointer is always
            // allowed. If `len` is zero, the slice pointer is convertable to a
            // reference since the pointer metadata is zero as well. The
            // dangling pointer is properly aligned.
            Box(NonNull::<[MaybeUninit<T>]>::slice_from_raw_parts(
                NonNull::<MaybeUninit<T>>::dangling(),
                len,
            ))
        } else {
            let Ok(layout) = Layout::array::<T>(len) else {
                panic!("slice length exceeded `isize::MAX`");
            };

            // SAFETY: Since both `size_of::<T>()` and `len` are checked for
            // zero, `layout` must have a non-zero size.
            let ptr = unsafe { alloc(layout) };

            let Some(ptr) = NonNull::new(ptr) else {
                handle_alloc_error(layout);
            };

            let ptr = NonNull::<[MaybeUninit<T>]>::slice_from_raw_parts(
                ptr.cast::<MaybeUninit<T>>(),
                len,
            );

            // SAFETY: `ptr` points to an allocation of an appropriate layout,
            // and `MaybeUninit` accepts potentially uninitialized memory. The
            // pointer will not be freed until `self` is dropped.
            Box::<[MaybeUninit<T>]>(ptr)
        }
    }
}

impl<T: ?Sized> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: `self.0` is guaranteed to be convertable to a pointer
        unsafe { self.0.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: `self.0` is guaranteed to be convertable to a pointer
        unsafe { self.0.as_mut() }
    }
}

impl<T: ?Sized> Drop for Box<T> {
    fn drop(&mut self) {
        // SAFETY: The pointer is convertable to a reference, we have exlusive
        // access, and it will never be used again
        unsafe { drop_in_place(self.0.as_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use crate::boxed::Box;

    const _: () = {
        assert!(size_of::<Box<i32>>() == size_of::<usize>());
        assert!(size_of::<Option<Box<i32>>>() == size_of::<usize>());
        assert!(size_of::<Box<()>>() == size_of::<usize>());
        assert!(size_of::<Box<str>>() == size_of::<usize>() * 2);
        assert!(size_of::<Option<Box<str>>>() == size_of::<usize>() * 2);
    };

    #[test]
    fn test_new() {
        let _ = Box::new(5);
    }

    #[test]
    fn test_into_inner() {
        let boxed = Box::new(5);
        assert_eq!(Box::into_inner(boxed), 5);
    }

    #[test]
    fn test_deref() {
        let boxed = Box::new(5);
        assert_eq!(&*boxed, &5);
    }

    #[test]
    fn test_deref_mut() {
        let mut boxed = Box::new(5);
        *boxed = 3;
        assert_eq!(&*boxed, &3);
    }

    #[test]
    fn test_drop() {
        struct Ty<'a> {
            drop_called: &'a mut bool,
        }

        impl<'a> Drop for Ty<'a> {
            fn drop(&mut self) {
                *self.drop_called = true;
            }
        }

        let mut drop_called = false;
        {
            let _ = Box::new(Ty {
                drop_called: &mut drop_called,
            });
        }

        assert!(drop_called);
    }
}
