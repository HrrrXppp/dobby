use nix::fcntl::OFlag;
use nix::sys::stat::{fstat, Mode};
use nix::sys::mman::{mmap, munmap, shm_open, shm_unlink, MapFlags, ProtFlags};
use std::os::unix::io::RawFd;
use nix::unistd::{ ftruncate, close };
use std::ptr::null_mut;


#[derive(Debug)]
pub struct SharedMemory {
    fd: RawFd,
    size: usize,
    ptr: *mut u8,
    is_owner: bool,   
    unique_id: String,
}

impl SharedMemory {
    pub fn new(unique_id: &str, size: usize) -> SharedMemory {
        println!("{}", unique_id);
        let shmem_fd = match shm_open(
            unique_id, 
            OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_RDWR, 
            Mode::S_IRUSR | Mode::S_IWUSR,                  
        ) {
            Ok(value) => value,
            Err(nix::Error::EEXIST) => return SharedMemory::open(unique_id),
            Err(e) => panic!("Shared memory didn't create, error in shm_open! {}", e),
        };
        
        match ftruncate(shmem_fd, size as i64) {
            Ok(_) => {}
            Err(e) => panic!("Shared memory didn't ftruncate! {}", e),
        };
    
        let ptr = match unsafe {
            mmap(
                null_mut(),                                   
                size,                             
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE, 
                MapFlags::MAP_SHARED,                         
                shmem_fd,                               
                0,                                            
            )
        } {
            Ok(value) => {value as *mut _}
            Err(e) => panic!("Shared memory didn't mmap! {}", e),
        };
    
        SharedMemory{fd: shmem_fd, size: size, ptr: ptr, is_owner: true, unique_id: unique_id.to_string()}
    }

    pub unsafe fn as_slice_mut(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self.as_ptr(), self.len())
    }

    pub unsafe fn as_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(self.as_ptr(), self.len())
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn resize(&mut self, size: usize) -> SharedMemory {
        self.drop();
        return SharedMemory::new(&self.unique_id, size);
    }

    pub fn is_owner(&self) -> bool {
        self.is_owner
    }

    fn open(unique_id: &str) -> SharedMemory {
        let shmem_fd = match shm_open(
            unique_id,
            OFlag::O_RDWR,
            Mode::S_IRUSR,
        ) {
            Ok(value) => {value},
            Err(e) => panic!("Shared memory didn't open, error in shm_open! {}", e),
        };
    
        let size = match fstat(shmem_fd) {
            Ok(value) => value.st_size as usize,
            Err(e) => panic!("Shared memory didn't fstat! {}", e),
        };
    
        let ptr = match unsafe {
            mmap(
                null_mut(),                                   
                size,                             
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE, 
                MapFlags::MAP_SHARED,                         
                shmem_fd,                               
                0,                                            
            )
        } {
            Ok(value) => {value as *mut _}
            Err(e) => panic!("Shared memory didn't mmap! {}", e),
        };
    
        SharedMemory{fd: shmem_fd, size: size, ptr: ptr, is_owner: false, unique_id: unique_id.to_string() }      
    }    

    pub fn drop(&mut self) {
        if !self.ptr.is_null() {
            if let Err(_e) = unsafe { munmap(self.ptr as *mut _, self.size) } {
                panic!("Failed to munmap() shared memory mapping : {}", _e);
            };
        }

        if self.fd != 0 {
            if let Err(_e) = shm_unlink(self.unique_id.as_str()) {
            };

            if let Err(_e) = close(self.fd) {
                panic!("Failed to close() shared memory file descriptor : {}",_e);
            };
        }
    }

} 
