use crate::access::{
    Access, Mut, NotAccessibleMut, NotAccessibleRef, RawMut, RawMutGuard, RawRef, RawRefGuard, Ref,
};
use std::cell::{Cell, UnsafeCell};
use std::marker;
use std::ptr::NonNull;

struct Inner<T: ?Sized> {
    access: Access,
    count: Cell<usize>,
    /// The value being held. Guarded by the `access` field to determine if it
    /// can be access shared or exclusively.
    data: UnsafeCell<T>,
}

/// A shared value.
pub struct Shared<T: ?Sized> {
    inner: NonNull<Inner<T>>,
}

impl<T> Shared<T> {
    /// Construct a new shared value.
    pub fn new(data: T) -> Self {
        let inner = Box::leak(Box::new(Inner {
            access: Access::new(),
            count: Cell::new(1),
            data: data.into(),
        }));

        Self {
            inner: inner.into(),
        }
    }

    /// Get a reference to the interior value while checking for shared access.
    pub fn get_ref(&self) -> Result<Ref<'_, T>, NotAccessibleRef> {
        // Safety: We know that interior value is alive since this container is
        // alive.
        //
        // Appropriate access is checked when constructing the guards.
        unsafe {
            let inner = self.inner.as_ref();
            inner.access.shared()?;

            Ok(Ref {
                raw: RawRef {
                    value: inner.data.get(),
                    guard: RawRefGuard {
                        access: &inner.access,
                    },
                },
                _marker: marker::PhantomData,
            })
        }
    }

    /// Get a reference to the interior value while checking for exclusive access.
    pub fn get_mut(&self) -> Result<Mut<'_, T>, NotAccessibleMut> {
        // Safety: We know that interior value is alive since this container is
        // alive.
        //
        // Appropriate access is checked when constructing the guards..
        unsafe {
            let inner = self.inner.as_ref();
            inner.access.exclusive()?;

            Ok(Mut {
                raw: RawMut {
                    value: inner.data.get(),
                    guard: RawMutGuard {
                        access: &inner.access,
                    },
                },
                _marker: marker::PhantomData,
            })
        }
    }
}

impl<T: ?Sized> Clone for Shared<T> {
    fn clone(&self) -> Self {
        unsafe {
            let count = self.inner.as_ref().count.get();

            if count == 0 || count == usize::max_value() {
                panic!("illegal count `{}` when cloning shared", count)
            }

            let count = count + 1;
            self.inner.as_ref().count.set(count);
        }

        Self { inner: self.inner }
    }
}

impl<T: ?Sized> Drop for Shared<T> {
    fn drop(&mut self) {
        unsafe {
            let count = self.inner.as_ref().count.get();

            if count == 0 {
                panic!("illegal count `{}` when dropping shared", count)
            }

            let count = count - 1;
            self.inner.as_ref().count.set(count);

            if count == 0 {
                // Safety: inner is guaranteed to be valid at the point we construct
                // the shared reference.
                let _ = Box::from_raw(self.inner.as_ptr());
            }
        }
    }
}
