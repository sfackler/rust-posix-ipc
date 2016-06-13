//! IPC semaphores.

use libc;

pub mod named;
pub mod unnamed;

struct RawSemaphore(*mut libc::sem_t);

impl RawSemaphore {
    fn wait(&self) {
        let r = unsafe { libc::sem_wait(self.0) };
        if r == libc::EDEADLK {
            panic!("semaphore wait would result in deadlock");
        }
        debug_assert_eq!(r, 0);
    }

    fn post(&self) {
        let r = unsafe { libc::sem_post(self.0) };
        debug_assert_eq!(r, 0);
    }
}
