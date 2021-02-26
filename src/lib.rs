use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

///
/// A special RwLock which can be locked exclusively any number of consecutive times,
///  but once initially locked shared, can never be unlocked.
/// This allows unguarded reads to occur
pub struct FusedRwLock<T: ?Sized> {
    inner: parking_lot::RwLock<()>,
    locked: AtomicBool,
    object: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for FusedRwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for FusedRwLock<T> {}

impl<T: Default> Default for FusedRwLock<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> FusedRwLock<T> {
    ///
    /// Constructs a new, initially unlocked, RwLock
    pub const fn new(x: T) -> Self {
        Self {
            inner: parking_lot::const_rwlock(()),
            locked: AtomicBool::new(false),
            object: UnsafeCell::new(x),
        }
    }

    ///
    /// Moves the inner value out of the FusedRwLock.
    /// This is sound because self is moved into the function, and thus no other accesses exist
    pub fn into_inner(self) -> T {
        self.object.into_inner()
    }
}

impl<T: ?Sized> FusedRwLock<T> {
    ///
    /// Mutably borrows the interior of the lock, if it has not been locked for reading access
    /// This is sound because taking self by &mut statically guarantees no other accesses exist.
    /// Returns None if the lock has been locked for reading
    pub fn try_get_mut(&mut self) -> Option<&mut T> {
        if *self.locked.get_mut() {
            Some(self.object.get_mut())
        } else {
            None
        }
    }

    ///
    /// Mutably borrows the interior of the lock, even if it has been locked for reading.
    /// This function is unsafe because, while not necessarily undefined behaviour, calling this function
    ///  after it was locked for reading can be used to violate the logical invariant of FusedRwLock.
    pub unsafe fn get_mut_unlocked(&mut self) -> &mut T {
        self.object.get_mut()
    }

    ///
    /// Check if the FusedRwLock has been locked for reading.
    /// This does not guarantee any synchronization, even if it returns true. Except where self is reborrowed from &mut,
    ///  it should only be used as a hint to avoid needless calls to self.try_read
    /// A return of true is guaranteed to remain true for the lifetime of the lock.
    /// A return of false may be invalidated at any time.
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    ///
    /// Locks this FusedRwLock for reading.
    /// After this call, it becomes impossible to acquire the lock for writing,
    ///  and safe code cannot be used to modify the inner value (except inside an UnsafeCell)
    pub fn lock(&self) {
        let _guard = self.inner.read();
        self.locked
            .store(true, std::sync::atomic::Ordering::Release)
    }

    ///
    /// Returns a shared reference to the interior of the lock, if it has been locked for reading.
    pub fn try_read(&self) -> Option<&T> {
        if self.locked.load(Ordering::Acquire) {
            // Safety:
            // Because self.locked is set, the lock can never be borrowed exclusively again
            //
            Some(unsafe { &*self.object.get() })
        } else {
            None
        }
    }

    pub fn read(&self) -> &T {
        if !self.is_locked() {
            self.lock();
        }
        self.try_read().unwrap()
    }

    pub fn try_write(&self) -> Option<FusedRwLockGuard<T>> {
        // Optimization, since a true return from self.is_locked is guaranteed to continue forever
        if !self.is_locked() {
            let guard = self.inner.write();
            if !self.is_locked() {
                Some(FusedRwLockGuard {
                    guard,
                    inner: unsafe { &mut *self.object.get() },
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct FusedRwLockGuard<'a, T: ?Sized> {
    guard: parking_lot::RwLockWriteGuard<'a, ()>,
    inner: &'a mut T,
}

impl<'a, T: ?Sized> Deref for FusedRwLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY:
        // self.guard ensures that self.cell can be borrowed
        self.inner
    }
}

impl<'a, T: ?Sized> DerefMut for FusedRwLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY:
        // self.guard ensures that self.cell can be borrowed
        self.inner
    }
}
