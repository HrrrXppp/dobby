use shared_memory::SharedMemory;

use libc::{pthread_mutex_t, pthread_mutex_trylock, pthread_mutex_unlock, EBUSY};
use core::cell::UnsafeCell;
use std::mem::size_of;
use nix::unistd::Pid;
use dobby_hash::get_hash;

pub struct Guard{
    mutex_ptr: *mut UnsafeCell<pthread_mutex_t>,
    shmem: SharedMemory,
}

impl Guard{
    pub fn create_lock(mutex_shmem_name: &str) -> Guard {
        
        println!("Guard create_lock. Pid: {}", Pid::this());

        let shmem = SharedMemory::new( get_hash( mutex_shmem_name ), size_of::<UnsafeCell<pthread_mutex_t>>() );
        let ptr = shmem.as_ptr() as *mut UnsafeCell<pthread_mutex_t>;

        let guard = Guard { mutex_ptr: ptr, shmem: shmem };

        unsafe{ guard.lock() };

        return guard;
    }

    unsafe fn lock(&self) {

        println!("Guard lock. Pid: {}", Pid::this());

        loop {
            match pthread_mutex_trylock( (*self.mutex_ptr).get() ) {
                0 => break,
                EBUSY => continue,
                _ => panic!("Failed to pthread_mutex_lock!")
            }
        }

        println!("Guard lock success! Pid: {}", Pid::this());

    }
}

impl Drop for Guard {
    fn drop(&mut self) {

        let res = unsafe{ pthread_mutex_unlock( (*self.mutex_ptr).get() ) };

        if res != 0 {
            panic!("Failed to pthread_mutex_unlock!");
        }

        println!("Guard drop. Pid: {}", Pid::this());

        //self.shmem.drop();
    }
}