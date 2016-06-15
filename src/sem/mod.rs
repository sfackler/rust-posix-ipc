//! IPC semaphores.

use libc;
use std::error::Error;
use std::fmt;
use std::io;

pub mod named;
pub mod unnamed;

struct RawSemaphore(*mut libc::sem_t);

impl RawSemaphore {
    fn wait(&self) {
        let r = unsafe { libc::sem_wait(self.0) };
        if r < 0 && io::Error::last_os_error().raw_os_error().unwrap() == libc::EDEADLK {
            panic!("semaphore wait would result in deadlock");
        }
        debug_assert_eq!(r, 0);
    }

    fn try_wait(&self) -> Result<(), TryWaitError> {
        let r = unsafe { libc::sem_trywait(self.0) };
        if r < 0 {
            match io::Error::last_os_error().raw_os_error().unwrap() {
                libc::EDEADLK => panic!("semaphore try_wait would result in deadlock"),
                libc::EAGAIN => Err(TryWaitError(())),
                e => {
                    debug_assert_eq!(e, 0);
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    fn post(&self) {
        let r = unsafe { libc::sem_post(self.0) };
        debug_assert_eq!(r, 0);
    }
}

/// An error returned when `try_wait` would have blocked.
#[derive(Debug)]
pub struct TryWaitError(());

impl fmt::Display for TryWaitError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

impl Error for TryWaitError {
    fn description(&self) -> &str {
        "wait call failed because the operation would block"
    }
}
