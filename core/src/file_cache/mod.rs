use crate::traits::{ WorkWithHashMap };
use crate::settings::Settings;

use shared_memory::*;
use guard::Guard;
use std::mem::size_of;
use std::fs::{File, read_dir, metadata, remove_file};
use std::io::Read;
use std::convert::{ TryInto };
use std::ptr;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use core::cell::UnsafeCell;
use libc::pthread_mutex_t;
use libc::PTHREAD_MUTEX_INITIALIZER;
use std::mem::MaybeUninit;
use dobby_hash::get_hash;

#[derive(Debug)]
struct CacheNode{
    // Simple, without collision, without rebalanced, for small node count
    hash: u64, 
    left: *mut CacheNode,
    right: *mut CacheNode,
    shared_memory: SharedMemory,
    load_time: Instant
}

impl CacheNode{

    fn create_node(&mut self, file_name_hash: u64, value: &String ) {
        let size = value.len();
        print!("create_node {}\n", size );
        self.shared_memory = SharedMemory::new( get_hash( &file_name_hash.to_string() ), value.as_bytes().len() );
        print!("create_node, create shared_memory\n");
        self.left = ptr::null_mut();
        self.right = ptr::null_mut();
        self.hash = file_name_hash;
        self.load_time = Instant::now();
        unsafe{ self.shared_memory.as_slice_mut().copy_from_slice( value.as_bytes() ) };
        print!("self.shared_memory.as_slice {}\n", unsafe{ self.shared_memory.to_string() } );
    }

    fn get_with_reload( &mut self, file_name_hash: u64, file_name: &str, reload_period: Duration ) -> Option< String > {
        print!("get_with_reload {:?} {:?}\n", file_name_hash, self.hash );
        if self.hash == file_name_hash {
            let result: String;
            println!( "self.load_time.elapsed() {:?}", self.load_time.elapsed() );
            println!( "reload_period {:?}", reload_period );
            if self.load_time.elapsed() < reload_period {
                result = self.get_data();                
            } else {
                let mut contents = String::new();
                let mut file =  match File::open( file_name ){
                    Ok( opened_file ) => opened_file,
                    Err( _ ) => return None
                };
                file.read_to_string( &mut contents ).unwrap();
                println!( "contents {}", contents );
                println!( "contents.as_bytes().len(): {}", contents.as_bytes().len() );
                println!( "self.shared_memory.len() {}", self.shared_memory.len() );                
                if contents.as_bytes().len() != self.shared_memory.len() {
                    self.shared_memory = self.shared_memory.resize(contents.as_bytes().len());            
                }                
                println!( "shared_memory.len() after resize {}", self.shared_memory.len() );                
                unsafe{ self.shared_memory.as_slice_mut().copy_from_slice( contents.as_bytes() ) };
                self.load_time = Instant::now();
                result = self.get_data();
            };
            print!("result {:?}\n", result );
            return Some( result );
        }

        let mut current_node: *mut CacheNode = ptr::null_mut();
        if file_name_hash < self.hash {
            current_node = self.left;
            print!("self.left\n");
        }
        else if file_name_hash > self.hash {
            current_node = self.right;
            print!("self.right\n");
        }
        match unsafe{ current_node.as_mut() } {
            Some( node ) => return node.get_with_reload( file_name_hash, file_name, reload_period ),
            _ => {}
        }
        print!("return None\n");
        return None;
    }

    pub fn get( &self, file_name_hash: u64 ) -> Option< String > {
        println!("get {:?} {:?}\n", file_name_hash, self.hash );
        if self.hash == file_name_hash {
            println!("self.hash == file_name_hash");
            println!("self {:?}", self );
            println!("self ptr {:p}", self);
            let result = self.get_data();
            print!("result {:?}\n", result );
            return Some( result );
        }
        println!("let mut current_node: *mut CacheNode = ptr::null_mut()");
        let mut current_node: *mut CacheNode = ptr::null_mut();
        if file_name_hash < self.hash {
            current_node = self.left;
            print!("self.left\n");
        }
        else if file_name_hash > self.hash {
            current_node = self.right;
            print!("self.right\n");
        }
        match unsafe{ current_node.as_ref() } {
            Some( node ) => return node.get( file_name_hash ),
            _ => {}
        }
        print!("return None\n");
        return None;
    }

    fn get_data( &self ) -> String {
        println!("CacheNode.get_data() self: {:?}", self);
        let res = unsafe{ self.shared_memory.to_string() };
        println!("End CacheNode.get_data()");
        return res;
    }

    pub fn drop(&mut self) {
        self.shared_memory.drop();
    }
}


pub struct FileCache{
    root_cache: *mut CacheNode,
    counter_mem: SharedMemory,
    node_count_mem: SharedMemory,
    tree_mem: SharedMemory,
    node_count_ptr: *mut i32,
    counter_ptr:  *mut i32,
    max_nodes: usize,
    settings: Settings,
}

impl FileCache{
    pub fn new( file_settings_name: &str ) -> FileCache {
        let new_settings = Settings::new( file_settings_name );

        let nodes_count = new_settings.get( "nodes_count" ).parse::<usize>().unwrap();

        let tree_mem_size = size_of::< CacheNode >() * nodes_count;
        println!( "tree_mem_size = {}, nodes_count = {}\n", tree_mem_size, nodes_count );
        let tree_shmem = SharedMemory::new( get_hash(&new_settings.get( "file_cache_name" )), tree_mem_size);
        let node_count_shmem = SharedMemory::new( get_hash(&new_settings.get( "file_cache_node_count_name" )), size_of::<i32>() );
        let counter_shmem = SharedMemory::new( get_hash(&new_settings.get( "file_cache_counter_name" )), size_of::<i32>() );        

        let root_cache_ptr = tree_shmem.as_ptr() as *mut CacheNode;
        let node_count_ptr = node_count_shmem.as_ptr() as *mut i32;
        let counter_ptr =  counter_shmem.as_ptr() as *mut i32;
        println!( "root_cache_ptr = {:?}, node_count_ptr = {:?}, node_count = {:?}", root_cache_ptr, node_count_ptr, unsafe{ *node_count_ptr } );

        if unsafe{ *counter_ptr } == 0 {
            let shmem = SharedMemory::new( get_hash(&new_settings.get( "mutex_name" )), size_of::<UnsafeCell<pthread_mutex_t>>() );
            let ptr = shmem.as_ptr() as *mut UnsafeCell<pthread_mutex_t>;            
            unsafe{ *ptr = UnsafeCell::new(PTHREAD_MUTEX_INITIALIZER) }  
            
            let mut attr = MaybeUninit::<libc::pthread_mutexattr_t>::uninit();
            if unsafe{ libc::pthread_mutexattr_init(attr.as_mut_ptr()) } != 0 {
                panic!("Failed pthread_mutexattr_init!")
            }

            struct PthreadMutexAttr<'a>(&'a mut MaybeUninit<libc::pthread_mutexattr_t>);

            let attr = PthreadMutexAttr(&mut attr);
            if unsafe{ libc::pthread_mutexattr_settype(attr.0.as_mut_ptr(), libc::PTHREAD_MUTEX_NORMAL) } != 0 {
                panic!("Failed pthread_mutexattr_settype!")
            }
            if unsafe{ libc::pthread_mutex_init((*ptr).get(), attr.0.as_ptr()) } != 0 {
                panic!("Failed pthread_mutex_init!")
            }
                  
        }

        unsafe{ *counter_ptr += 1 };        
        println!( "counter_ptr {}", unsafe{ *counter_ptr }  );


        return FileCache{ 
            counter_ptr: counter_ptr,
            root_cache: root_cache_ptr,
            counter_mem: counter_shmem,
            tree_mem: tree_shmem,
            node_count_mem: node_count_shmem,
            node_count_ptr: node_count_ptr,
            max_nodes: nodes_count,
            settings: new_settings,
        };
    }

    fn add_node( &mut self, new_node: *mut CacheNode ) {
        let root_node: *mut CacheNode = self.root_cache;
        print!( "new_node {:?}\n", new_node );
        print!( "*self.node_count_ptr {:?}\n", unsafe{*self.node_count_ptr} );
        match unsafe{ *self.node_count_ptr } {
            0 => {
                self.root_cache = new_node;
                print!( "self.root_cache {:?}\n", unsafe{ self.root_cache.as_ref().unwrap() } );
            },
            _ => {
                let mut current_ptr: *mut CacheNode = root_node;
                print!( "new_node {:?}\n", new_node );
                loop {
                    let ref mut current = unsafe{ current_ptr.as_mut().unwrap() };
                    let ref new = unsafe{ new_node.as_ref().unwrap() };
                    let current_hash = current.hash;
                    let new_node_hash = new.hash;
                    if current_hash == new_node_hash {
                        panic!( "Hash Collision  {}", current_hash );
                    }                    
                    else if current_hash > new_node_hash {
                        current_ptr = current.left;
                        print!("current.left\n");
                    }                
                    else if current_hash < new_node_hash {
                        current_ptr = current.right;
                        print!("current.right\n");
                    }                
                    match unsafe{ current_ptr.as_ref() } {
                        None => {
                            print!("current_ptr = new_node\n");
                            if current_hash > new_node_hash {
                                current.left = new_node;    
                            }
                            else {
                                current.right = new_node; 
                            }
                            return;
                        },
                        _ => {}                     
                    } 
                }
            }
        }
    }

    fn load_file_to_cashe( &mut self, file_name: &str ) -> Option< String > {
        let ref mut count: usize = ( unsafe{ *self.node_count_ptr } ).try_into().unwrap();
        println!( "load_file_to_cashe {} node_count {}\n", file_name, count );
        if *count >= self.max_nodes {
            panic!( "Maximum number of nodes reached {}", self.max_nodes );
        }
        let mut contents = String::new();
        let mut file =  match File::open( file_name ){
            Ok( opened_file ) => opened_file,
            Err( _ ) => return None
        };
        file.read_to_string( &mut contents ).unwrap();
        print!( "contents {}", contents );
        let file_name_hash = get_hash( file_name );
        let node_ptr: *mut CacheNode = unsafe{ self.root_cache.offset( ( *count ).try_into().unwrap() ) };
        print!( "load_file_to_cashe node_ptr = {:?}\n", node_ptr );
        unsafe{ &mut *node_ptr }.create_node( file_name_hash, &contents );
        self.add_node( unsafe{ &mut *node_ptr } );
        unsafe{ *self.node_count_ptr += 1 };
        return Some( contents );
    }

    pub fn get_file_with_reload( &mut self, file_name: &str, reload_period: Duration ) -> Option< String > {
        return self.get_from_cache_with_reload( file_name, Some(reload_period) );
    }

    pub fn get_file( &mut self, file_name: &str ) -> Option< String > {
        println!( "get_file {}", file_name );
        return self.get_from_cache_with_reload( file_name, None );
    }

    fn get_from_cache_with_reload( &mut self, file_name: &str, reload_period: Option<Duration> ) -> Option<String> {        
        let _lock = Guard::create_lock( &self.settings.get("mutex_name" ));
        println!( "get_from_cache_with_reload. *self.node_count_ptr {:?}", unsafe{ *self.node_count_ptr } );
        let file_name_hash = get_hash( file_name );
        let mut res = match unsafe{ *self.node_count_ptr } {
            0 => None,
            _ =>  match reload_period {
                None => unsafe{ self.root_cache.as_mut() }.unwrap().get( file_name_hash ),
                _ => unsafe{ self.root_cache.as_mut() }.unwrap().get_with_reload( file_name_hash, file_name, reload_period.unwrap() )
            }
        };          
        if None == res {      
            res = self.load_file_to_cashe( file_name )    
        }
        println!("res: {:?}", res);
        return res;
    }

    pub fn reset(path: &str) {
        let paths = read_dir( path ).unwrap();

        for path in paths {
            let ref _path = path.unwrap().path();
            let metadata = metadata(_path).unwrap();
            if ! metadata.is_file() {
                continue;
            }
            if &(_path.file_name().unwrap().to_str().unwrap()[..6]) == SHARED_MEMORY_PREFIX {
                remove_file(_path).unwrap();
            }
        }
    }
}

impl Drop for FileCache {
    fn drop(&mut self) {
        println!( "file cache counter before {}", unsafe{ *self.counter_ptr } );
        unsafe{ *self.counter_ptr -= 1 };        
        println!( "file cache counter after {}", unsafe{ *self.counter_ptr } );
        if  unsafe{ *self.counter_ptr == 0 && *self.node_count_ptr > 0 } {
            let root_node = unsafe{ self.root_cache.as_mut() };
            let mut queue: VecDeque<Option<&mut CacheNode>> = VecDeque::new();
            queue.push_back( root_node );
            loop {
                if queue.len() == 0 {
                    break;
                } 
                let current_node = queue.pop_front();
                match current_node {
                    Some ( node ) => {
                        match unsafe{ node.as_ref().unwrap().left.as_mut() } {
                            Some( node_left ) => {
                                queue.push_back( Some( node_left ) );
                            },
                            _ => {}
                        };
                        match unsafe{ node.as_ref().unwrap().right.as_mut() } {
                            Some( node_right ) => {
                                queue.push_back( Some( node_right ) );
                            },
                            _ => {}
                        };
                        let cache_node = node.unwrap();
                        cache_node.drop();
                    },
                    _ => {}        
                }
            }
            self.tree_mem.drop();    
            self.counter_mem.drop();
            self.node_count_mem.drop();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::file_cache::FileCache; 
    use crate::traits::Process;
    use std::thread;   
    use std::io::Write;

    #[test]
    fn get_one_file() {
        println!("*************** get_one_file ****************");
        let mut file_cache = FileCache::new( "../file_cache.cfg" );
        let file_404: String = file_cache.get_file( "../404.html" ).unwrap();
        let file_hello: String = file_cache.get_file( "../source/hello.html" ).unwrap();
        let worker_cfg: String = file_cache.get_file( "../worker.cfg" ).unwrap();
        let file_cache_cfg: String = file_cache.get_file( "../file_cache.cfg" ).unwrap();

        let handle = thread::spawn(move || {
            let mut thread_file_cache = FileCache::new( "../file_cache.cfg" );
            let thread_file_404: String = thread_file_cache.get_file( "../404.html" ).unwrap();
            assert_eq!(thread_file_404, file_404);
            let thread_file_hello: String = thread_file_cache.get_file( "../source/hello.html" ).unwrap();
            assert_eq!(thread_file_hello, file_hello);
            let thread_worker_cfg: String = thread_file_cache.get_file( "../worker.cfg" ).unwrap();
            assert_eq!(thread_worker_cfg, worker_cfg);
            let thread_file_cache_cfg: String = thread_file_cache.get_file( "../file_cache.cfg" ).unwrap();
            assert_eq!(thread_file_cache_cfg, file_cache_cfg);
            });    
        handle.join().unwrap();
        let _file_hello_1: String = file_cache.get_file( "../source/hello.html" ).unwrap();
        let _worker_cfg_1: String = file_cache.get_file( "../worker.cfg" ).unwrap();
        let _file_40_14: String = file_cache.get_file( "../404.html" ).unwrap();
        let _file_cache_cfg_1: String = file_cache.get_file( "../file_cache.cfg" ).unwrap();
        println!("---------------- get_one_file -----------------");
    }

    #[test]

    fn get_file_with_reload() {
        println!("*************** get_file_with_reload ****************");
        let file_name = "/tmp/test_get_file_with_reload.txt";
        let test_text_1 = "1111111111111111";
        let test_text_2 = "111111111";
        let test_text_3 = "111111111444444444444444";
        let mut file_cache = FileCache::new( "../file_cache.cfg" );

        let mut output1 = std::fs::File::create(file_name).unwrap();
        write!( output1, "{}", test_text_1).unwrap();
        let test_file_1: String = file_cache.get_file( file_name ).unwrap();
        assert_eq!(test_text_1, test_file_1);

        let mut output2 = std::fs::File::create(file_name).unwrap();
        write!( output2, "{}", test_text_2).unwrap();
        let test_file_2: String = file_cache.get_file_with_reload( file_name, core::time::Duration::new(0, 0) ).unwrap();
        assert_eq!(test_text_2, test_file_2);

        let mut output3 = std::fs::File::create(file_name).unwrap();
        write!( output3, "{}", test_text_3).unwrap();
        let test_file_3: String = file_cache.get_file_with_reload( file_name, core::time::Duration::new(0, 0) ).unwrap();
        assert_eq!(test_text_3, test_file_3);

        println!("---------------- get_file_with_reload -----------------");
    }

    #[test]
    fn test_mutex(){
        struct TestProcess {
            file_cache: Option<FileCache>
        }
        
        impl Process for TestProcess {
            fn init( &mut self ) {
                self.file_cache = Some( FileCache::new( "../file_cache.cfg" ) );        
            }
        
            fn run( &mut self ){

                for _num in 0..100 {
                   let _file_hello_1: String = self.file_cache.as_mut().unwrap().get_file( "../source/hello.html" ).unwrap();
                }                
            }
        }

        FileCache::reset( "/dev/shm/" );
        let mut tp_1 = TestProcess{file_cache: None};
        let _pid_1 = tp_1.create();
        let _pid_2 = tp_1.create();
    }
}
