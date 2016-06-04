use libc;

use std::ffi::{OsStr, CString};
use std::io;
use std::mem;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

pub struct SharedMemory(libc::c_int);

impl Drop for SharedMemory {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
        }
    }
}

impl SharedMemory {
    pub fn set_len(&self, size: u64) -> io::Result<()> {
        unsafe {
            if size > libc::off_t::max_value() as u64 {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "size too large"));
            }

            if libc::ftruncate(self.0, size as libc::off_t) == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

impl AsRawFd for SharedMemory {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl IntoRawFd for SharedMemory {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.0;
        mem::forget(self);
        fd
    }
}

impl FromRawFd for SharedMemory {
    unsafe fn from_raw_fd(fd: RawFd) -> SharedMemory {
        SharedMemory(fd)
    }
}

pub struct OpenOptions {
    write: bool,
    create: bool,
    create_new: bool,
    truncate: bool,
    mode: libc::mode_t,
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            write: false,
            create: false,
            create_new: false,
            truncate: false,
            mode: 0o666,
        }
    }

    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        self.write = write;
        self
    }

    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        self.create = create;
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions {
        self.create_new = create_new;
        self
    }

    pub fn mode(&mut self, mode: u32) -> &mut OpenOptions {
        self.mode = mode as libc::mode_t;
        self
    }

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
            let ret = libc::shm_open(name.as_ptr(), oflag, self.mode as libc::c_int);
            if ret >= 0 {
                Ok(SharedMemory(ret))
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

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
