use libc;
use std::io;

use sem::RawSemaphore;

pub struct Semaphore(RawSemaphore);

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            libc::sem_destroy((self.0).0);
        }
    }
}

impl Semaphore {
    pub unsafe fn new(sem: *mut libc::sem_t, value: u32) -> io::Result<Semaphore> {
        if libc::sem_init(sem, 1, value as libc::c_uint) == 0 {
            Ok(Semaphore(RawSemaphore(sem)))
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn wait(&self) {
        self.0.wait()
    }

    pub fn post(&self) {
        self.0.post()
    }
}

pub struct SemaphoreRef(RawSemaphore);

impl SemaphoreRef {
    pub unsafe fn new(sem: *mut libc::sem_t) -> SemaphoreRef {
        SemaphoreRef(RawSemaphore(sem))
    }

    pub fn wait(&self) {
        self.0.wait()
    }

    pub fn post(&self) {
        self.0.post()
    }
}
