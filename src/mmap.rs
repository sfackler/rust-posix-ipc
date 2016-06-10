use libc;
use std::io;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::slice;
use std::os::unix::io::AsRawFd;

pub struct MemoryMap {
    base: *mut libc::c_void,
    len: usize,
}

impl Drop for MemoryMap {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.base, self.len);
        }
    }
}

impl Deref for MemoryMap {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.base as *const u8, self.len)
        }
    }
}

impl DerefMut for MemoryMap {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self.base as *mut u8, self.len)
        }
    }
}

pub struct MapOptions {
    shared: bool,
    fixed: bool,
    addr: *mut libc::c_void,
    read: bool,
    write: bool,
    exec: bool,
}

impl MapOptions {
    pub fn new() -> MapOptions {
        MapOptions {
            shared: false,
            fixed: false,
            addr: ptr::null_mut(),
            read: false,
            write: false,
            exec: false,
        }
    }

    pub fn shared(&mut self, shared: bool) -> &mut MapOptions {
        self.shared = shared;
        self
    }

    pub fn fixed(&mut self, fixed: bool) -> &mut MapOptions {
        self.fixed = fixed;
        self
    }

    pub fn addr(&mut self, addr: *mut libc::c_void) -> &mut MapOptions {
        self.addr = addr;
        self
    }

    pub fn read(&mut self, read: bool) -> &mut MapOptions {
        self.read = read;
        self
    }

    pub fn write(&mut self, write: bool) -> &mut MapOptions {
        self.write = write;
        self
    }

    pub fn exec(&mut self, exec: bool) -> &mut MapOptions {
        self.exec = exec;
        self
    }

    fn map_inner(&self,
                 len: usize,
                 mut flags: libc::c_int,
                 fd: libc::c_int,
                 offset: u64) -> io::Result<MemoryMap> {
        let mut prot = libc::PROT_NONE;
        if self.read {
            prot |= libc::PROT_READ;
        }
        if self.write {
            prot |= libc::PROT_WRITE;
        }
        if self.exec {
            prot |= libc::PROT_EXEC;
        }

        if self.shared {
            flags |= libc::MAP_SHARED;
        } else {
            flags |= libc::MAP_PRIVATE;
        }
        if self.fixed {
            flags |= libc::MAP_FIXED;
        }

        let offset = if offset > libc::off_t::max_value() as u64 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "offset too large"));
        } else {
            offset as libc::off_t
        };

        unsafe {
            let addr = libc::mmap(self.addr, len, prot, flags, fd, offset);
            if addr == libc::MAP_FAILED {
                Err(io::Error::last_os_error())
            } else {
                Ok(MemoryMap {
                    base: addr,
                    len: len,
                })
            }
        }
    }

    pub unsafe fn map<T: AsRawFd>(&self, len: usize, fd: &T, offset: u64) -> io::Result<MemoryMap> {
        self.map_inner(len, libc::MAP_FILE, fd.as_raw_fd(), offset)
    }

    pub fn map_anonymous(&self, len: usize) -> io::Result<MemoryMap> {
        self.map_inner(len, libc::MAP_ANON, 0, 0)
    }
}
