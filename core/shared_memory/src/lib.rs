use nix::fcntl::OFlag;
use nix::sys::stat::{fstat, Mode};
use nix::sys::mman::{mmap, munmap, shm_open, shm_unlink, MapFlags, ProtFlags};
use std::os::unix::io::RawFd;
use nix::unistd::{ ftruncate, close };
use std::ptr::null_mut;
use nix::libc::c_char;
use std::ffi::CStr;
use std::str;

pub static SHARED_MEMORY_PREFIX: &str = "dobby.";

#[derive(Debug)]
pub struct SharedMemory {
    fd: RawFd,
    size: usize,
    ptr: *mut u8,
    unique_id: u64,
}

impl SharedMemory {

    fn get_full_id(unique_id: u64) -> String {
        return SHARED_MEMORY_PREFIX.to_owned() + &unique_id.to_string();
    }

    pub fn new(unique_id: u64, size: usize) -> SharedMemory {
        
        let full_unique_id = SharedMemory::get_full_id(unique_id);

        println!("unique_id: {}", unique_id);

        let shmem_fd = match shm_open(
            full_unique_id.as_str(), 
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
    
        SharedMemory{fd: shmem_fd, size: size, ptr: ptr, unique_id: unique_id}
    }

    pub unsafe fn as_slice_mut(&mut self) -> &mut [u8] {
        println!("SharedMemory.as_slice_mut()");
        std::slice::from_raw_parts_mut(self.as_ptr(), self.len())
    }

    pub fn to_string(&self) -> String {
        println!("SharedMemory.as_slice()");
        let ptr = self.as_ptr();
        let len = self.len();
        println!("ptr: {:?}, len: {}", ptr, len);
        let slice = unsafe{ std::slice::from_raw_parts(ptr, len) };
        println!("1");
        let str_buf = str::from_utf8( slice );
        println!("SharedMemory.as_slice() end!!! res: {:?}", str_buf);
        return str_buf.unwrap().to_string();
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn resize(&mut self, size: usize) -> SharedMemory {
        self.drop();
        return SharedMemory::new(self.unique_id, size);
    }    

    fn open(unique_id: u64) -> SharedMemory {

        let full_unique_id = SharedMemory::get_full_id(unique_id);

        let shmem_fd = match shm_open(
            full_unique_id.as_str(),
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
    
        SharedMemory{fd: shmem_fd, size: size, ptr: ptr, unique_id: unique_id }      
    }    

    pub fn drop(&mut self) {
        if !self.ptr.is_null() {
            if let Err(_e) = unsafe { munmap(self.ptr as *mut _, self.size) } {
                panic!("Failed to munmap() shared memory mapping : {}", _e);
            };
        }

        if self.fd != 0 {

            let full_unique_id = SharedMemory::get_full_id(self.unique_id);

            if let Err(_e) = shm_unlink(full_unique_id.as_str()) {
            };

            if let Err(_e) = close(self.fd) {
                panic!("Failed to close() shared memory file descriptor : {}",_e);
            };
        }
    }
} 
