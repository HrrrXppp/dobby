pub mod registry;

use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::sync::Mutex;

#[macro_use]
extern crate lazy_static;


type FuncType = fn(&str) -> String;

lazy_static! {
    static ref FUNCS: Mutex<HashMap<String, FuncType>> =  Mutex::new( HashMap::new());
}

#[no_mangle]
pub extern "C" fn run( method: &str, args: &str ) -> String {
    println!( "method, args {} {}", method, args);
    return "Hello, world!".to_string();
}

fn source( filename: &str ) -> String {
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string( &mut contents ).unwrap();
    return contents;
}

fn main() {    
    FUNCS.lock().unwrap().insert( "source".to_string(), source );
}
