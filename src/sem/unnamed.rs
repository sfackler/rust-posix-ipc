//! Unnamed IPC semaphores.

use libc;
use std::io;

use sem::{RawSemaphore, TryWaitError};

/// An owned, unnamed, IPC semaphore.
pub struct Semaphore(RawSemaphore);

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            libc::sem_destroy((self.0).0);
        }
    }
}

impl Semaphore {
    /// Initializes an unnamed semaphore.
    ///
    /// The semaphore is configured for use by multiple processes.
    ///
    /// Some platforms such as OSX and FreeBSD do not support unnamed
    /// semaphores, and this function will always return an error.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that the memory pointed to by
    /// `sem` remains valid for the lifetime of the returned object.
    pub unsafe fn new(sem: *mut libc::sem_t, value: u32) -> io::Result<Semaphore> {
        if libc::sem_init(sem, 1, value as libc::c_uint) == 0 {
            Ok(Semaphore(RawSemaphore(sem)))
        } else {
            Err(io::Error::last_os_error())
        }
    }

    /// Decrements the semaphore by 1, blocking if semaphore's value is 0.
    pub fn wait(&self) {
        self.0.wait()
    }

    /// Attempts to decrement the semaphore by 1, returning an error if the
    /// semaphore's value is 0.
    pub fn try_wait(&self) -> Result<(), TryWaitError> {
        self.0.try_wait()
    }

    /// Increments the semaphore by 1.
    pub fn post(&self) {
        self.0.post()
    }
}

/// An unowned, unnamed, IPC semaphore.
pub struct SemaphoreRef(RawSemaphore);

impl SemaphoreRef {
    /// Creates a new `SemaphoreRef` referencing a previously initialized
    /// unnamed semaphore.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that the memory pointed to by
    /// `sem` references a valid, initialized unnamed semaphore for the
    /// lifetime of the returned object.
    pub unsafe fn new(sem: *mut libc::sem_t) -> SemaphoreRef {
        SemaphoreRef(RawSemaphore(sem))
    }

    /// Decrements the semaphore by 1, blocking if semaphore's value is 0.
    pub fn wait(&self) {
        self.0.wait()
    }

    /// Increments the semaphore by 1.
    pub fn post(&self) {
        self.0.post()
    }
}
