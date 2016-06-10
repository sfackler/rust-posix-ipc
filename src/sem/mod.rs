use libc;
use std::io;

pub mod named;
pub mod unnamed;

struct RawSemaphore(*mut libc::sem_t);

impl RawSemaphore {
    fn wait(&self) -> io::Result<()> {
        unsafe {
            if libc::sem_wait(self.0) == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    fn post(&self) -> io::Result<()> {
        unsafe {
            if libc::sem_post(self.0) == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}
