//! POSIX shared memory

use libc;

use std::ffi::{OsStr, CString};
use std::io;
use std::fs::{File, Metadata};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

// shm_open is variadic on OSX but not anything else so we have to do this :(
#[cfg(target_os = "macos")]
type ModeHack = libc::c_int;
#[cfg(not(target_os = "macos"))]
type ModeHack = libc::mode_t;

/// A shared memory region.
pub struct SharedMemory(File);

impl SharedMemory {
    /// Sets the length of the shared memory region.
    ///
    /// If `size` is greater than the current size of the region, it will be
    /// extended with 0s.
    pub fn set_len(&self, size: u64) -> io::Result<()> {
        self.0.set_len(size)
    }

    /// Returns metadata about the shared memory region, including its length.
    pub fn metadata(&self) -> io::Result<Metadata> {
        self.0.metadata()
    }

    /// Returns a new independently owned handle to the same shared memory
    /// region.
    pub fn try_clone(&self) -> io::Result<SharedMemory> {
        self.0.try_clone().map(SharedMemory)
    }
}

impl AsRawFd for SharedMemory {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoRawFd for SharedMemory {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for SharedMemory {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> SharedMemory {
        SharedMemory(File::from_raw_fd(fd))
    }
}

/// A builder type for `SharedMemory`.
pub struct OpenOptions {
    write: bool,
    create: bool,
    create_new: bool,
    truncate: bool,
    mode: libc::mode_t,
}

impl OpenOptions {
    /// Returns a new `OpenOptions` with default settings.
    pub fn new() -> OpenOptions {
        OpenOptions {
            write: false,
            create: false,
            create_new: false,
            truncate: false,
            mode: 0o666,
        }
    }

    /// Sets the option for write access.
    ///
    /// The shared memory region is writable if and only if this option is set.
    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        self.write = write;
        self
    }

    /// Sets the option for creating a new shared memory region.
    ///
    /// This option indicates whether a new region will be created if it does
    /// not already exist.
    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        self.create = create;
        self
    }

    /// Sets the option to always create a new shared memory region.
    ///
    /// This option indicates whether a new region will be created. If the
    /// region already exists, the operation will fail. This check happens
    /// atomically.
    ///
    /// If this option is set, the value of `create` is ignored.
    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions {
        self.create_new = create_new;
        self
    }

    /// Sets the access mode use when creating a new shared memory region.
    ///
    /// If the region already exists, this is ignored.
    pub fn mode(&mut self, mode: u32) -> &mut OpenOptions {
        self.mode = mode as libc::mode_t;
        self
    }

    /// Opens a shared memory region.
    ///
    /// The name should begin with a `/` and contain no further `/`s.
    pub fn open<T: AsRef<OsStr>>(&self, name: T) -> io::Result<SharedMemory> {
        let name = try!(CString::new(name.as_ref().as_bytes()));

        let mut oflag = 0;
        if self.write {
            oflag |= libc::O_RDWR;
        } else {
            oflag |= libc::O_RDONLY;
        }
        if self.create {
            oflag |= libc::O_CREAT;
        }
        if self.create_new {
            oflag |= libc::O_EXCL | libc::O_CREAT;
        }
        if self.truncate {
            oflag |= libc::O_TRUNC;
        }

        unsafe {
            let ret = libc::shm_open(name.as_ptr(), oflag, self.mode as ModeHack);
            if ret >= 0 {
                Ok(SharedMemory::from_raw_fd(ret))
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

/// Removes a shared memory region.
pub fn unlink<T: AsRef<OsStr>>(name: T) -> io::Result<()> {
    unsafe {
        let name = try!(CString::new(name.as_ref().as_bytes()));
        if libc::shm_unlink(name.as_ptr()) == 0 {
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
    fn metadata() {
        let name = "/posix-pic-shm-metadata";
        let shm = OpenOptions::new().create(true).write(true).open(name).unwrap();
        unlink(name).unwrap();
        shm.set_len(4096).unwrap();
        assert_eq!(shm.metadata().unwrap().len(), 4096);
    }
}
