//! Named IPC semaphores.

use libc;
use std::io;
use std::ffi::{OsStr, CString};
use std::os::unix::ffi::OsStrExt;

use sem::RawSemaphore;

/// A POSIX named semaphore.
pub struct Semaphore(RawSemaphore);

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            libc::sem_close((self.0).0);
        }
    }
}

impl Semaphore {
    /// Opens an existing IPC semaphore.
    ///
    /// The name must begin with a `/` and contain no further `/`s.
    pub fn open<T: AsRef<OsStr>>(name: T) -> io::Result<Semaphore> {
        OpenOptions::new().open(name.as_ref())
    }

    /// Opens an IPC semaphore, creating it if it does not already exist.
    ///
    /// The name must begin with a `/` and contain no further `/`s.
    pub fn create<T: AsRef<OsStr>>(name: T) -> io::Result<Semaphore> {
        OpenOptions::new().create(true).open(name.as_ref())
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

/// A builder for `Semaphore`s.
pub struct OpenOptions {
    create: bool,
    create_new: bool,
    mode: libc::mode_t,
    value: libc::c_uint,
}

impl OpenOptions {
    /// Creates a new `OpenOptions` with default settings.
    pub fn new() -> OpenOptions {
        OpenOptions {
            create: false,
            create_new: false,
            mode: 0o666,
            value: 0,
        }
    }

    /// Sets the option for creating a new semaphore.
    ///
    /// This option indicates whether a new semaphore will be created if it does
    /// not already exist.
    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        self.create = create;
        self
    }

    /// Sets the option to always create a new semaphore.
    ///
    /// This option indicates whether a new semaphore will be created. If the
    /// semaphore already exists, the operation will fail. This check happens
    /// atomically.
    ///
    /// If this option is set, the value of `create` is ignored.
    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions {
        self.create_new = create_new;
        self
    }

    /// Sets the access mode use when creating a new semaphore.
    ///
    /// If the semaphore already exists, this is ignored.
    pub fn mode(&mut self, mode: u32) -> &mut OpenOptions {
        self.mode = mode as libc::mode_t;
        self
    }

    /// Sets the initial value of the semaphore.
    ///
    /// If the semaphore already exists, this is ignored.
    pub fn value(&mut self, value: u32) -> &mut OpenOptions {
        self.value = value as libc::c_uint;
        self
    }

    /// Opens a named semaphore.
    ///
    /// The name must begin with a `/` and contain no further `/`s.
    pub fn open<T: AsRef<OsStr>>(&self, name: T) -> io::Result<Semaphore> {
        let name = try!(CString::new(name.as_ref().as_bytes()));

        let mut flags = 0;
        if self.create {
            flags |= libc::O_CREAT;
        }
        if self.create_new {
            flags |= libc::O_EXCL | libc::O_CREAT;
        }

        unsafe {
            let sem = libc::sem_open(name.as_ptr(), flags, self.mode as libc::c_int, self.value);
            if sem == libc::SEM_FAILED {
                Err(io::Error::last_os_error())
            } else {
                Ok(Semaphore(RawSemaphore(sem)))
            }
        }
    }
}

/// Removes a named semaphore.
pub fn unlink<T: AsRef<OsStr>>(name: T) -> io::Result<()> {
    let name = try!(CString::new(name.as_ref().as_bytes()));

    unsafe {
        if libc::sem_unlink(name.as_ptr()) == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_thread() {
        let name = "/posix-ipc-sem-single-thread";
        let sem = OpenOptions::new().create_new(true).open(name).unwrap();
        unlink(name).unwrap();
        sem.post();
        sem.wait();
    }

    #[test]
    fn open_missing() {
        let name = "/posix-ipc-sem-open-missing";
        assert!(Semaphore::open(name).is_err());
    }

    #[test]
    fn create_open() {
        let name = "/posix-ipc-sem-create-open";
        Semaphore::create(name).unwrap();
        Semaphore::open(name).unwrap();
        unlink(name).unwrap();
    }
}
