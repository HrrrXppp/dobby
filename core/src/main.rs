pub mod registry;
pub mod common_struct;

use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::sync::Mutex;
use common_struct::Params;
extern crate rustc_serialize;
use rustc_serialize::json;

#[macro_use]
extern crate lazy_static;


type FuncType = fn( &Vec< Params > ) -> String;

lazy_static! {
    static ref FUNCS: Mutex<HashMap<String, FuncType>> =  Mutex::new( HashMap::new());
}

#[no_mangle]
pub extern "Rust" fn run( method: &str, args: &str ) -> String {
    println!( "method, args {} {}", method, args);
    let params: Vec< Params > = match json::decode::< Vec::< Params > >( args ) {
        Ok( res ) => res,
        _ => panic!( "Can't decode params!" )
    };
    println!( "FUNCS.lock().unwrap().len() {:?}", FUNCS.lock().unwrap().len() );
    println!( "params {:?}", params);
    return FUNCS.lock().unwrap().get( method ).unwrap()( &params );
}

#[no_mangle]
pub extern "Rust" fn init() {
    FUNCS.lock().unwrap().insert( "send_file".to_string(), get_source );
}

fn get_source( params: &Vec< Params > ) -> String {
    if params.len() < 1 {
        panic!( "Call get_source without filename" );
    }
    let file_name: &str = params[ 0 ].get_as_str();    
    println!( "file_name {:?}", params);
    let mut file = File::open( file_name ).unwrap();
    let mut contents = String::new();
    file.read_to_string( &mut contents ).unwrap();
    return contents;
}

fn main() {    
}
