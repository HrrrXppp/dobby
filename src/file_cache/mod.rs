use crate::traits::{ WorkWithHashMap };
use crate::settings::Settings;

use shared_memory::*;
use raw_sync::locks::*;
use std::mem::size_of;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fs::File;
use std::io::Read;
use std::convert::{ TryInto };
use std::ptr;


struct CasheNode{
    // Simple, without collision, without rebalanced, cor small node count
    hash: u64, 
    left: *mut CasheNode,
    right: *mut CasheNode,
    shared_mem: Shmem
}

impl CasheNode{

    fn create_node(&mut self, file_name_hash: u64, value: &String ) {
        let size = value.len();
        print!("create_node {}\n", size );
        self.shared_mem = match ShmemConf::new().size( size ).flink( file_name_hash.to_string() ).create() {
            Ok( m)  => m,
            Err( ShmemError::LinkExists ) => panic!( "Shared memory exist {}", file_name_hash ),
            Err( e ) => panic!( "Don't create shared memory {}", e ),
        };
        print!("create_node, create shared_mem\n");
        self.left = ptr::null_mut();
        self.right = ptr::null_mut();
        self.hash = file_name_hash;
//        unsafe{ self.shared_mem.as_ptr().write_bytes( value[..].as_ptr() as u8, size ) };
        unsafe{ self.shared_mem.as_slice_mut().clone_from_slice( value.as_bytes() ) };
        print!("self.shared_mem.as_slice {}\n", unsafe{ String::from_utf8_lossy( self.shared_mem.as_slice() ) } );
    }

    pub fn get( &self, file_name_hash: u64 ) -> Option< String > {        
        print!("get {:?} {:?}\n", file_name_hash, self.hash );
        if self.hash == file_name_hash {
            let result = self.get_data();
            print!("result {:?}\n", result );
            return Some( result );
        }

        let mut current_node: *mut CasheNode = ptr::null_mut();
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
        return unsafe{ String::from_utf8_lossy( self.shared_mem.as_slice() ) }.to_string();
    }
}

pub struct FileCache{
    mutex: Box::< dyn raw_sync::locks::LockImpl >,
    root_cache: *mut *mut CasheNode,
    mutex_mem: Shmem,
    tree_mem: Shmem,
    node_count: *mut u64,
    nodes: *mut CasheNode,
    max_nodes: usize
}

impl FileCache{
    pub fn new( file_settings_name: &str ) -> FileCache {
        let new_settings = Settings::new( file_settings_name );
        let mutex_name: String = new_settings.get( "mutex_name" );
        let mutex_shmem = match ShmemConf::new().size( 4096 ).flink( &mutex_name ).create() {
            Ok( m)  => m,
            Err( ShmemError::LinkExists ) =>ShmemConf::new().flink( &mutex_name ).open().unwrap(),
            Err( e ) => panic!( "Don't create shared memory {}", e ),
        };
        
        let temp_mutex: Box::< dyn raw_sync::locks::LockImpl >;
        let base_ptr = mutex_shmem.as_ptr();

        if mutex_shmem.is_owner(){
            temp_mutex = unsafe{
                Mutex::new( base_ptr, base_ptr.add( Mutex::size_of( Some( base_ptr ) ) ) ).unwrap().0 
            };
        }
        else{
            temp_mutex = unsafe {
                Mutex::from_existing( base_ptr, base_ptr.add( Mutex::size_of( Some( base_ptr ) ) ) ).unwrap().0
            };    
        }
        let tree_shmem: Shmem;
        let nodes_count: usize;
        {
            let mut _guard = temp_mutex.lock().unwrap();

            let file_cache_name: String = new_settings.get( "file_cache_name" );
            nodes_count = new_settings.get( "nodes_count" ).parse::<usize>().unwrap();
            let tree_mem_size = size_of::< CasheNode >() * nodes_count + size_of::< *mut *mut CasheNode >() + size_of::< *mut u64 >();
            print!( "tree_mem_size = {}, nodes_count = {}\n", tree_mem_size, nodes_count );
            tree_shmem = match ShmemConf::new().size( tree_mem_size ).flink( &file_cache_name ).create() {
                Ok( m)  => m,
                Err( ShmemError::LinkExists ) =>ShmemConf::new().flink( &file_cache_name ).open().unwrap(),
                Err( e ) => panic!( "Don't create shared memmory {}", e ),
            };
        }
        let root_cache_ptr = tree_shmem.as_ptr() as *mut *mut CasheNode;
        let node_count_ptr = unsafe{ tree_shmem.as_ptr().offset( size_of::< *mut *mut CasheNode >() .try_into().unwrap() ) as *mut u64 };
        let tree_ptr = unsafe{ tree_shmem.as_ptr().offset( (size_of::< *mut *mut CasheNode >() + size_of::< *mut u64 >()).try_into().unwrap() ) as *mut CasheNode };
        print!( "node_count_ptr = {:?}, tree_ptr = {:?}\n", node_count_ptr, tree_ptr );
        return FileCache{ 
            mutex: temp_mutex,
            root_cache: root_cache_ptr,
            mutex_mem: mutex_shmem,
            tree_mem: tree_shmem,
            node_count: node_count_ptr,
            nodes: tree_ptr,
            max_nodes: nodes_count,
        };
    }

    fn add_node( &self, new_node: *mut CasheNode ) {
        let root_node: *mut CasheNode = unsafe{ *self.root_cache };
        print!( "new_node {:?}\n", new_node );
        match unsafe{ root_node.as_ref() } {
            Some( _ ) => {
                let mut current_ptr: *mut CasheNode = root_node;
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
            },
            None => {
                unsafe{ *self.root_cache = new_node };
                print!( "self.root_cache {:?}\n", unsafe{ self.root_cache.as_ref().unwrap() } );
            }
        }
    }

    fn load_file_to_cashe( &mut self, file_name: &str ) -> String {
        let ref mut count: usize = ( unsafe{ *self.node_count } ).try_into().unwrap();
        print!( "load_file_to_cashe {} {}\n", file_name, count );
        if *count >= self.max_nodes {
            panic!( "Maximum number of nodes reached {}", self.max_nodes );
        }
        let mut contents = String::new();
        let mut file = File::open( file_name ).unwrap();
        file.read_to_string( &mut contents ).unwrap();
        print!( "contents {}", contents );
        let file_name_hash = self.get_hash( file_name );
        let node_ptr: *mut CasheNode = unsafe{ self.nodes.offset( ( *count ).try_into().unwrap() ) };
        print!( "load_file_to_cashe node_ptr = {:?}\n", node_ptr );
        unsafe{ &mut *node_ptr }.create_node( file_name_hash, &contents );
        unsafe{ *self.node_count += 1 };
        self.add_node( unsafe{ &mut *node_ptr } );
        return contents;
    }

    pub fn get_file( &mut self, file_name: &str ) -> String {
        let res = match self.get_from_cache( file_name ){
            Some( data ) => data,
            None => self.load_file_to_cashe( file_name )
        };

        return res;
    }

    fn get_hash( &self, file_name: &str ) -> u64 {
        let mut hasher = DefaultHasher::new();
        file_name.hash(&mut hasher);
        return hasher.finish();
    }

    fn get_from_cache( &self, file_name: &str ) -> Option<String> {        
        print!( "get_from_cache {:?} \n", unsafe{ self.root_cache.as_ref().unwrap() } );
        match unsafe{ self.root_cache.as_ref().unwrap().as_ref() } {
            Some( node ) => {
                let mut _guard = self.mutex.lock().unwrap();
                let file_name_hash = self.get_hash( file_name );
                print!( "file_name_hash {:?} \n", file_name_hash );
                return node.get( file_name_hash );        
            },
            _ => {}
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use crate::file_cache::FileCache; 
    use std::thread;   
    #[test]
    fn get_file() {
        let mut file_cache = FileCache::new( "file_cache.cfg" );

        let file_hello: String = file_cache.get_file( "hello.html" );
        let worker_cfg: String = file_cache.get_file( "worker.cfg" );
        let file_404: String = file_cache.get_file( "404.html" );
        let file_cache_cfg: String = file_cache.get_file( "file_cache.cfg" );

        let handle = thread::spawn(move || {
            let thread_file_cache = FileCache::new( "file_cache.cfg" );
            let thread_file_404: String = thread_file_cache.get_from_cache( "404.html" ).unwrap();
            assert_eq!(thread_file_404, file_404);
            let thread_file_hello: String = thread_file_cache.get_from_cache( "hello.html" ).unwrap();
            assert_eq!(thread_file_hello, file_hello);
            let thread_worker_cfg: String = thread_file_cache.get_from_cache( "worker.cfg" ).unwrap();
            assert_eq!(thread_worker_cfg, worker_cfg);
            let thread_file_cache_cfg: String = thread_file_cache.get_from_cache( "file_cache.cfg" ).unwrap();
            assert_eq!(thread_file_cache_cfg, file_cache_cfg);
            });    
        handle.join().unwrap();
    }
}
