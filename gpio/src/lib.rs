use core::settings;
use core::file_cache;
use core::common_struct::Params;
use core::traits::WorkWithHashMap;

use std::collections::HashMap;
use std::sync::Mutex;
extern crate rustc_serialize;
use rustc_serialize::json;
use std::time::{Duration};

type FuncType = fn( &Vec< Params > ) -> String;

static mut FUNCS: Option<Mutex<HashMap<String, FuncType>>> = None;//  Mutex::new( HashMap::new());
static mut SETTINGS: Option<settings::Settings>  = None;
static mut FILE_CACHE: Option<file_cache::FileCache>  = None;//file_cache::FileCache::new("file_name");

#[no_mangle]
pub extern "Rust" fn run( method: &str, args: &str ) -> String {
    println!( "method, args {} {}", method, args);
    let params: Vec< Params > = match json::decode::< Vec::< Params > >( args ) {
        Ok( res ) => res,
        _ => panic!( "Can't decode params!" )
    };
    println!( "FUNCS.lock().unwrap().len() {:?}", unsafe{ FUNCS.as_ref().unwrap().lock().unwrap().len() } ) ;
    println!( "params {:?}", params);
    return unsafe{ FUNCS.as_ref().unwrap().lock().unwrap().get( method ).unwrap()( &params ) };
}

#[no_mangle]
pub extern "Rust" fn init() {
    unsafe{ FUNCS = Some( Mutex::new( HashMap::new() ) ) }; 
    unsafe{ SETTINGS = Some( settings::Settings::new( "worker.cfg" )) }
    unsafe{ FILE_CACHE = Some( file_cache::FileCache::new( &SETTINGS.as_ref().unwrap().get( "file_cache_setting_file_name") )) }
    unsafe{ FUNCS.as_ref().unwrap().lock().unwrap().insert( "get_w1".to_string(), get_w1 ) };
}

fn get_w1( params: &Vec< Params > ) -> String {
    if params.len() < 1 {
        panic!( "Call get_w1 without device name" );
    }
    let device_name: &str = params[ 0 ].get_as_str();    
    println!( "file_name {:?}", device_name);
    let result = match unsafe{ FILE_CACHE.as_mut().unwrap().get_file_with_reload( &("/sys/bus/w1/devices/".to_owned() + device_name + "/w1_slave" ), Duration::new(300, 0) ) } {
        Some( res ) => res,
        None => match unsafe{ FILE_CACHE.as_mut().unwrap().get_file( &( SETTINGS.as_ref().unwrap().get( "error_404_file_name"  ) ) ) } {
            Some( res ) => res,
            None => panic!( "Not found 404 error file" )
        }
    };
    return result;
}
