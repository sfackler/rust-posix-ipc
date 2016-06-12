extern crate libc;

pub mod mmap;
pub mod sem;
pub mod shm;

#[cfg(test)]
mod test {
    use shm;
    use mmap;
    use sem::unnamed::{Semaphore, SemaphoreRef};

    #[test]
    #[cfg_attr(target_os = "macos", ignore)]
    fn unnamed_sem() {
        let name = "/posix-ipc-unnamed-sem";
        let shm = shm::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(name)
            .unwrap();
        shm::unlink(name).unwrap();
        shm.set_len(4096).unwrap();

        unsafe {
            let mut mmap = mmap::MapOptions::new()
                .read(true)
                .write(true)
                .shared(true)
                .map(4096, &shm, 0)
                .unwrap();

            let sem_ptr = mmap.as_mut_ptr() as *mut _;
            let _sem = Semaphore::new(sem_ptr, 1).unwrap();

            let sem_ref = SemaphoreRef::new(sem_ptr);
            sem_ref.wait();
        }
    }
}
