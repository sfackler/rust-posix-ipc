//! Mapped memory.

use libc;
use std::io;
use std::ptr;
use std::os::unix::io::AsRawFd;

/// A mapped memory region.
pub struct MemoryMap {
    base: *mut libc::c_void,
    len: usize,
}

unsafe impl Send for MemoryMap {}
unsafe impl Sync for MemoryMap {}

impl Drop for MemoryMap {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.base, self.len);
        }
    }
}

impl MemoryMap {
    /// Returns a pointer to the base of this memory region.
    #[inline]
    pub fn as_ptr(&self) -> *const libc::c_void {
        self.base as *const _
    }

    /// Returns a mutable pointer to the base of this memory region.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut libc::c_void {
        self.base
    }

    /// Returns the length of this memory region.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

/// A builder type for `MemoryMap`s.
pub struct MapOptions {
    shared: bool,
    fixed: bool,
    addr: *mut libc::c_void,
    read: bool,
    write: bool,
    exec: bool,
}

impl MapOptions {
    /// Creates a new `MapOptions` with default settings.
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

    /// Sets the option to share changes to the memory region.
    ///
    /// If set, changes made to the mapped memory region will be reflected in
    /// the underlying object.
    pub fn shared(&mut self, shared: bool) -> &mut MapOptions {
        self.shared = shared;
        self
    }

    /// Sets the option to force the memory region to be mapped in a specific
    /// location.
    ///
    /// If set, the system must respect the value set by the `addr` method.
    pub fn fixed(&mut self, fixed: bool) -> &mut MapOptions {
        self.fixed = fixed;
        self
    }

    /// Sets the requested base address for the mapped region.
    ///
    /// Unless `fixed` is set, this address is treated as a hint.
    pub fn addr(&mut self, addr: *mut libc::c_void) -> &mut MapOptions {
        self.addr = addr;
        self
    }

    /// Sets the option for read access.
    ///
    /// If set, the mapped region will be readable.
    pub fn read(&mut self, read: bool) -> &mut MapOptions {
        self.read = read;
        self
    }

    /// Sets the option for write access.
    ///
    /// The mapped region is writable if and only if this option is set.
    pub fn write(&mut self, write: bool) -> &mut MapOptions {
        self.write = write;
        self
    }

    /// Sets the option for execute access.
    ///
    /// If set, the mapped region will be executable.
    pub fn exec(&mut self, exec: bool) -> &mut MapOptions {
        self.exec = exec;
        self
    }

    fn map_inner(&self,
                 len: usize,
                 mut flags: libc::c_int,
                 fd: libc::c_int,
                 offset: u64)
                 -> io::Result<MemoryMap> {
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

    /// Maps the object referenced by a file descriptor into memory.
    ///
    /// The file descriptor may correspond to a file, a shared memory region,
    /// or a typed memory region.
    pub fn map<T: AsRawFd>(&self, len: usize, fd: &T, offset: u64) -> io::Result<MemoryMap> {
        self.map_inner(len, libc::MAP_FILE, fd.as_raw_fd(), offset)
    }

    /// Maps a region of anonymous memory.
    pub fn map_anonymous(&self, len: usize) -> io::Result<MemoryMap> {
        self.map_inner(len, libc::MAP_ANON, 0, 0)
    }
}
