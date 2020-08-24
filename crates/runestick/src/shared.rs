use crate::access::{
    Access, Mut, NotAccessibleMut, NotAccessibleRef, RawMutGuard, RawRefGuard, Ref,
};
use crate::any::Any;
use crate::vm::VmError;
use std::any;
use std::cell::{Cell, UnsafeCell};
use std::fmt;
use std::marker;
use std::mem;
use std::ops;
use std::process;
use std::ptr::NonNull;
use thiserror::Error;

/// Error raised when tried to access for exclusively owned access.
#[derive(Debug, Error)]
#[error("not accessible for taking")]
pub struct NotOwned(());

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
}

impl<T> Shared<T> {
    /// Take the interior value, if we have exlusive access to it and there
    /// exist no other references.
    pub fn take(self) -> Result<T, NotOwned> {
        // Safety: We know that interior value is alive since this container is
        // alive.
        //
        // Appropriate access is checked when constructing the guards..
        unsafe {
            if !self.inner.as_ref().access.is_exclusive() || self.inner.as_ref().count.get() != 1 {
                return Err(NotOwned(()));
            }

            // NB: we need to prevent the Drop impl for Shared from being called,
            // since we are deconstructing its internals.
            let this = mem::ManuallyDrop::new(self);

            let inner = Box::from_raw(this.inner.as_ptr());
            Ok(inner.data.into_inner())
        }
    }
}

impl<T> Shared<T>
where
    T: any::Any,
{
    /// Get a reference to the interior value while checking for shared access
    /// that holds onto a reference count of the inner value.
    pub fn strong_ref(self) -> Result<StrongRef<T>, NotAccessibleRef> {
        // Safety: We know that interior value is alive since this container is
        // alive.
        //
        // Appropriate access is checked when constructing the guards.
        unsafe {
            let guard = self.inner.as_ref().access.shared()?;

            // NB: we need to prevent the Drop impl for Shared from being called,
            // since we are deconstructing its internals.
            let this = mem::ManuallyDrop::new(self);

            Ok(StrongRef {
                data: this.inner.as_ref().data.get(),
                guard,
                inner: RawInner::from_inner(this.inner),
                _marker: marker::PhantomData,
            })
        }
    }

    /// Get a reference to the interior value while checking for exclusive
    /// access that holds onto a reference count of the inner value.
    pub fn strong_mut(self) -> Result<StrongMut<T>, NotAccessibleMut> {
        // Safety: We know that interior value is alive since this container is
        // alive.
        //
        // Appropriate access is checked when constructing the guards.
        unsafe {
            let guard = self.inner.as_ref().access.exclusive()?;

            // NB: we need to prevent the Drop impl for Shared from being called,
            // since we are deconstructing its internals.
            let this = mem::ManuallyDrop::new(self);

            Ok(StrongMut {
                data: this.inner.as_ref().data.get(),
                guard,
                inner: RawInner::from_inner(this.inner),
                _marker: marker::PhantomData,
            })
        }
    }
}

impl<T: ?Sized> Shared<T> {
    /// Get a reference to the interior value while checking for shared access.
    pub fn get_ref(&self) -> Result<Ref<'_, T>, NotAccessibleRef> {
        // Safety: We know that interior value is alive since this container is
        // alive.
        //
        // Appropriate access is checked when constructing the guards.
        unsafe {
            let guard = self.inner.as_ref().access.shared()?;

            Ok(Ref {
                value: self.inner.as_ref().data.get(),
                guard,
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
            let guard = self.inner.as_ref().access.exclusive()?;

            Ok(Mut {
                value: self.inner.as_ref().data.get(),
                guard,
                _marker: marker::PhantomData,
            })
        }
    }
}

impl Shared<Any> {
    /// Take the interior value, if we have exlusive access to it and there
    /// exist no other references.
    pub fn downcast_take<T>(self) -> Result<T, VmError>
    where
        T: any::Any,
    {
        // Safety: We know that interior value is alive since this container is
        // alive.
        //
        // Appropriate access is checked when constructing the guards..
        unsafe {
            if !self.inner.as_ref().access.is_exclusive() || self.inner.as_ref().count.get() != 1 {
                return Err(VmError::from(NotOwned(())));
            }

            // NB: we need to prevent the Drop impl for Shared from being called,
            // since we are deconstructing its internals.
            let this = mem::ManuallyDrop::new(self);

            let inner = Box::from_raw(this.inner.as_ptr());
            let any = inner.data.into_inner();

            match any.take_mut_ptr(any::TypeId::of::<T>()) {
                Ok(value) => Ok(*Box::from_raw(value as *mut T)),
                Err(any) => {
                    return Err(VmError::UnexpectedValueType {
                        actual: any.type_name(),
                        expected: any::type_name::<T>(),
                    });
                }
            }
        }
    }

    /// Get a shared value and downcast.
    pub fn downcast_ref<T>(&self) -> Result<Ref<'_, T>, VmError>
    where
        T: any::Any,
    {
        unsafe {
            let guard = self.inner.as_ref().access.shared()?;

            let data = match (*self.inner.as_ref().data.get()).as_ptr(any::TypeId::of::<T>()) {
                Some(data) => data,
                None => {
                    return Err(VmError::UnexpectedValueType {
                        expected: any::type_name::<T>(),
                        actual: (*self.inner.as_ref().data.get()).type_name(),
                    });
                }
            };

            Ok(Ref {
                value: data as *const T,
                guard,
                _marker: marker::PhantomData,
            })
        }
    }

    /// Get a shared value and downcast.
    pub fn downcast_strong_ref<T>(self) -> Result<StrongRef<T>, VmError>
    where
        T: any::Any,
    {
        unsafe {
            let guard = self.inner.as_ref().access.shared()?;

            let data = match (*self.inner.as_ref().data.get()).as_ptr(any::TypeId::of::<T>()) {
                Some(data) => data,
                None => {
                    return Err(VmError::UnexpectedValueType {
                        expected: any::type_name::<T>(),
                        actual: (*self.inner.as_ref().data.get()).type_name(),
                    });
                }
            };

            // NB: we need to prevent the Drop impl for Shared from being called,
            // since we are deconstructing its internals.
            let this = mem::ManuallyDrop::new(self);

            Ok(StrongRef {
                data: data as *const T,
                guard,
                inner: RawInner::from_inner(this.inner),
                _marker: marker::PhantomData,
            })
        }
    }

    /// Get a exclusive value and downcast.
    pub fn downcast_mut<T>(&self) -> Result<Mut<'_, T>, VmError>
    where
        T: any::Any,
    {
        unsafe {
            let guard = self.inner.as_ref().access.exclusive()?;

            let data = match (*self.inner.as_ref().data.get()).as_mut_ptr(any::TypeId::of::<T>()) {
                Some(data) => data,
                None => {
                    return Err(VmError::UnexpectedValueType {
                        expected: any::type_name::<T>(),
                        actual: (*self.inner.as_ref().data.get()).type_name(),
                    });
                }
            };

            Ok(Mut {
                value: data as *mut T,
                guard,
                _marker: marker::PhantomData,
            })
        }
    }

    /// Get a shared value and downcast.
    pub fn downcast_strong_mut<T>(self) -> Result<StrongMut<T>, VmError>
    where
        T: any::Any,
    {
        unsafe {
            let guard = self.inner.as_ref().access.exclusive()?;

            let data = match (*self.inner.as_ref().data.get()).as_mut_ptr(any::TypeId::of::<T>()) {
                Some(data) => data,
                None => {
                    return Err(VmError::UnexpectedValueType {
                        expected: any::type_name::<T>(),
                        actual: (*self.inner.as_ref().data.get()).type_name(),
                    });
                }
            };

            // NB: we need to prevent the Drop impl for Shared from being called,
            // since we are deconstructing its internals.
            let this = mem::ManuallyDrop::new(self);

            Ok(StrongMut {
                data: data as *mut T,
                guard,
                inner: RawInner::from_inner(this.inner),
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
                process::abort();
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
                process::abort();
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

impl<T: ?Sized> fmt::Debug for Shared<T>
where
    T: any::Any + fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            if self.inner.as_ref().access.is_shared() {
                write!(fmt, "Shared(inaccessible: {})", any::type_name::<T>())?;
            } else {
                fmt.debug_tuple("Shared")
                    .field(&&*self.inner.as_ref().data.get())
                    .finish()?;
            }
        }

        Ok(())
    }
}

struct Inner<T: ?Sized> {
    /// The access of the shared data.
    access: Access,
    /// The number of strong references to the shared data.
    count: Cell<usize>,
    /// The value being held. Guarded by the `access` field to determine if it
    /// can be access shared or exclusively.
    data: UnsafeCell<T>,
}

type DropFn = unsafe fn(*const ());

struct RawInner {
    data: *const (),
    drop_fn: DropFn,
}

impl RawInner {
    /// Construct a raw inner from an existing inner value.
    ///
    /// # Safety
    ///
    /// Should only be constructed over a pointer that is lively owned.
    fn from_inner<T>(inner: NonNull<Inner<T>>) -> Self
    where
        T: any::Any,
    {
        return Self {
            data: inner.as_ptr() as *const (),
            drop_fn: drop_fn_impl::<T>,
        };

        unsafe fn drop_fn_impl<T>(data: *const ()) {
            let inner = data as *mut () as *mut Inner<T>;
            let count = (*inner).count.get();

            if count == 0 {
                process::abort();
            }

            let count = count - 1;
            (*inner).count.set(count);

            if count == 0 {
                let _ = Box::from_raw(inner);
            }
        }
    }
}

impl Drop for RawInner {
    fn drop(&mut self) {
        // Safety: type and referential safety is guaranteed at construction
        // time, since all constructors are unsafe.
        unsafe {
            (self.drop_fn)(self.data);
        }
    }
}

/// A strong reference to the given type.
pub struct StrongRef<T: ?Sized> {
    data: *const T,
    guard: RawRefGuard,
    inner: RawInner,
    _marker: marker::PhantomData<T>,
}

impl<T: ?Sized> StrongRef<T> {
    /// Convert into a raw pointer and associated raw access guard.
    ///
    /// # Safety
    ///
    /// The returned pointer must not outlive the associated guard, since this
    /// prevents other uses of the underlying data which is incompatible with
    /// the current.
    ///
    /// The returned pointer also must not outlive the VM that produced.
    /// Nor a call to clear the VM using [clear], since this will free up the
    /// data being referenced.
    ///
    /// [clear]: [crate::Vm::clear]
    pub fn into_raw(this: Self) -> (*const T, RawStrongRefGuard) {
        let guard = RawStrongRefGuard {
            _guard: this.guard,
            _inner: this.inner,
        };

        (this.data, guard)
    }
}

impl<T: ?Sized> ops::Deref for StrongRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T: ?Sized> fmt::Debug for StrongRef<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, fmt)
    }
}

/// A raw guard to a [StrongRef].
pub struct RawStrongRefGuard {
    _guard: RawRefGuard,
    _inner: RawInner,
}

/// A strong mutable reference to the given type.
pub struct StrongMut<T: ?Sized> {
    data: *mut T,
    guard: RawMutGuard,
    inner: RawInner,
    _marker: marker::PhantomData<T>,
}

impl<T: ?Sized> StrongMut<T> {
    /// Convert into a raw pointer and associated raw access guard.
    ///
    /// # Safety
    ///
    /// The returned pointer must not outlive the associated guard, since this
    /// prevents other uses of the underlying data which is incompatible with
    /// the current.
    ///
    /// The returned pointer also must not outlive the VM that produced.
    /// Nor a call to clear the VM using [clear], since this will free up the
    /// data being referenced.
    ///
    /// [clear]: [crate::Vm::clear]
    pub fn into_raw(this: Self) -> (*mut T, RawStrongMutGuard) {
        let guard = RawStrongMutGuard {
            _guard: this.guard,
            _inner: this.inner,
        };

        (this.data, guard)
    }
}

impl<T: ?Sized> ops::Deref for StrongMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T: ?Sized> ops::DerefMut for StrongMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<T: ?Sized> fmt::Debug for StrongMut<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, fmt)
    }
}

/// A raw guard to a [StrongRef].
pub struct RawStrongMutGuard {
    _guard: RawMutGuard,
    _inner: RawInner,
}
