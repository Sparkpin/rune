use std::cell::Cell;
use std::fmt;
use std::marker;
use std::ops;
use thiserror::Error;

/// Error raised when tried to access for shared access but it was not
/// accessible.
#[derive(Debug, Error)]
#[error("not accessible for shared access")]
pub struct NotAccessibleRef(());

/// Error raised when tried to access for exclusive access but it was not
/// accessible.
#[derive(Debug, Error)]
#[error("not accessible for exclusive access")]
pub struct NotAccessibleMut(());

#[derive(Debug, Clone)]
pub(crate) struct Access(Cell<isize>);

impl Access {
    /// Construct a new default access.
    pub(crate) const fn new() -> Self {
        Self(Cell::new(0))
    }

    /// Test if we have shared access without modifying the internal count.
    #[inline]
    pub(crate) fn is_shared(&self) -> bool {
        self.0.get().wrapping_sub(1) < 0
    }

    /// Test if we have exclusive access without modifying the internal count.
    #[inline]
    pub(crate) fn is_exclusive(&self) -> bool {
        self.0.get() == 0
    }

    /// Mark that we want shared access to the given access token.
    #[inline]
    pub(crate) fn shared(&self) -> Result<RawRefGuard, NotAccessibleRef> {
        let b = self.0.get().wrapping_sub(1);

        if b >= 0 {
            return Err(NotAccessibleRef(()));
        }

        self.0.set(b);

        Ok(RawRefGuard { access: self })
    }

    /// Mark that we want exclusive access to the given access token.
    #[inline]
    pub(crate) fn exclusive(&self) -> Result<RawMutGuard, NotAccessibleMut> {
        let b = self.0.get().wrapping_add(1);

        if b != 1 {
            return Err(NotAccessibleMut(()));
        }

        self.0.set(b);
        Ok(RawMutGuard { access: self })
    }

    /// Unshare the current access.
    #[inline]
    fn release_shared(&self) {
        let b = self.0.get().wrapping_add(1);
        debug_assert!(b <= 0);
        self.0.set(b);
    }

    /// Unshare the current access.
    #[inline]
    fn release_exclusive(&self) {
        let b = self.0.get().wrapping_sub(1);
        debug_assert!(b == 0);
        self.0.set(b);
    }
}

/// A raw reference guard.
pub(crate) struct RawRefGuard {
    access: *const Access,
}

impl Drop for RawRefGuard {
    fn drop(&mut self) {
        unsafe { (*self.access).release_shared() };
    }
}

/// Guard for a value borrowed from a slot in the virtual machine.
///
/// These guards are necessary, since we need to guarantee certain forms of
/// access depending on what we do. Releasing the guard releases the access.
///
/// These also aid in function call integration, since they can be "arm" the
/// virtual machine to release shared guards through its unsafe functions.
///
/// See [clear] for more information.
///
/// [clear]: [crate::Vm::clear]
pub struct Ref<'a, T: ?Sized + 'a> {
    pub(crate) value: *const T,
    pub(crate) guard: RawRefGuard,
    pub(crate) _marker: marker::PhantomData<&'a T>,
}

impl<'a, T: ?Sized> Ref<'a, T> {
    /// Try to map the interior reference the reference.
    pub fn try_map<M, U: ?Sized, E>(this: Self, m: M) -> Result<Ref<'a, U>, E>
    where
        M: FnOnce(&T) -> Result<&U, E>,
    {
        let value = m(unsafe { &*this.value })?;
        let guard = this.guard;

        Ok(Ref {
            value,
            guard,
            _marker: marker::PhantomData,
        })
    }
}

impl<T: ?Sized> ops::Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}

impl<T: ?Sized> fmt::Debug for Ref<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, fmt)
    }
}

/// A raw mutable guard.
pub(crate) struct RawMutGuard {
    access: *const Access,
}

impl Drop for RawMutGuard {
    fn drop(&mut self) {
        unsafe { (*self.access).release_exclusive() }
    }
}

/// Guard for a value exclusively borrowed from a slot in the virtual machine.
///
/// These guards are necessary, since we need to guarantee certain forms of
/// access depending on what we do. Releasing the guard releases the access.
///
/// These also aid in function call integration, since they can be "arm" the
/// virtual machine to release shared guards through its unsafe functions.
///
/// See [clear][crate::Vm::clear] for more information.
pub struct Mut<'a, T: ?Sized> {
    pub(crate) value: *mut T,
    pub(crate) guard: RawMutGuard,
    pub(crate) _marker: marker::PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> Mut<'a, T> {
    /// Map the mutable reference.
    pub fn try_map<M, U: ?Sized, E>(this: Self, m: M) -> Result<Mut<'a, U>, E>
    where
        M: FnOnce(&mut T) -> Result<&mut U, E>,
    {
        let value = m(unsafe { &mut *this.value })?;
        let guard = this.guard;

        Ok(Mut {
            value,
            guard,
            _marker: marker::PhantomData,
        })
    }
}

impl<T: ?Sized> ops::Deref for Mut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}

impl<T: ?Sized> ops::DerefMut for Mut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.value }
    }
}

impl<T: ?Sized> fmt::Debug for Mut<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, fmt)
    }
}
