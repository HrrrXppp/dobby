use core::common_struct::Params;

use std::collections::{ HashMap };
use std::result::{ Result };
use crate::traits::WorkWithHashMap;
extern crate libloading;
extern crate rustc_serialize;
use rustc_serialize::json;

type RunFunc = unsafe fn( &str, &str ) -> String;
type InitFunc = unsafe fn();

pub struct Executor{
    func_desc_map: HashMap< String, String>,
    func_map: HashMap< String, libloading::Library >,
}

impl Executor{
    fn split_action( action: &str ) -> ( &str, &str ) {
        let offset = action.find( "." ).unwrap_or( action.len() );
        let ( first, mut last) = action.split_at(offset);
        last = &last[1..];
        return ( first, last );
    }

    fn split_method( method: &str ) -> ( &str, &str ) {
        let offset = method.find( '(' ).unwrap_or( method.len() );
        let ( first, mut last) = method.split_at(offset);
        last = &last[1..];
        let offset1 = last.find( ')' ).unwrap_or( last.len() );
        let ( first1, _last1 ) = last.split_at(offset1);
        return ( first, first1.trim() );
    }

    fn create_args<'life_time>( args_desc: &'life_time str, args: &'life_time str ) -> Result<String, &'life_time str> {
        let desc_vec: Vec<&str> = args_desc.split( ',' ).collect();
        let args_vec: Vec<&str> = args.split( ',' ).collect();
        if desc_vec.len() != args_vec.len() {
            return Err( "Description arguments and arguments have different size!" );
        }
        let mut res_args : Vec< Params > = Vec::new();
        for ( i, desc ) in desc_vec.iter().enumerate() {
            let arg = args_vec[ i ];
            println!( "{} element has value {} {}", i, desc, arg );
            res_args.push( Params{ desc: desc.to_string(), arg: arg.to_string() } );
        }
        return Ok( json::encode(&res_args).unwrap() );
    }

    fn get_params<'lf>( &self, action: &'lf str, args: &'lf str ) -> ( &'lf str, &'lf str, String ) {
        let ( lib, method ) = Executor::split_action( action );
        let ( real_method, args_desc ) = Executor::split_method( method );
        let res_args = Executor::create_args( args_desc, args );
        print!( "res_args {:?}\n", res_args );
        match res_args {
            Ok(res_args_str) => return ( lib, real_method, res_args_str ),
            Err(e) => println!("Error in Executor::create_args: {}", e)
        }
        print!( "get_params before return\n" );
        return ( lib, real_method, "".to_string() );
    }

    fn get_lib( & mut self, lib_name: &str ) -> Option<& libloading::Library > {
        let mut opt_lib :Option<& libloading::Library > = None;
        if self.func_map.contains_key( lib_name ) {
            opt_lib = self.func_map.get( lib_name );
        }
        else
        {
            println!( "Loading library {}", lib_name );
            let lib = libloading::Library::new( "lib".to_string() + lib_name + ".so" );
            match lib {
                Ok( lib_unw ) => {
                    let init_func: libloading::Symbol<InitFunc> = unsafe{ lib_unw.get( b"init" ).unwrap() };
                    unsafe{ init_func() };
                    self.func_map.insert( lib_name.to_string(), lib_unw );
                    opt_lib = self.func_map.get( lib_name );
                },
                Err( e ) => println!("Unable load library: {} {}", lib_name, e)
            }
        }
        return opt_lib;
    }

    pub fn run( &mut self, action: &str, args: &str ) -> Option< String > {
        let ( lib_name, method, real_args ) = self.get_params( action, args );
        let lib = self.get_lib( lib_name );
        match lib {
            Some( real_lib ) => {
                unsafe {
                    let run_func: libloading::Symbol<RunFunc> = real_lib.get( b"run" ).unwrap();
                    return Some( run_func( method, &real_args ) );
                }        
            },
            None => {}
        }
        return None
    }
}

impl WorkWithHashMap for Executor{
    fn new<'lf>( filename: &str ) ->Executor {
        let mut new_executor = Executor{ func_desc_map: HashMap::new(), func_map: HashMap::new() };
        new_executor.load( filename );
        return new_executor;
    }

    fn get_hash_map( &self ) -> &HashMap<String, String>{
        return &self.func_desc_map;
    }

    fn get_mut_hash_map( &mut self ) -> &mut HashMap<String, String> {
        return &mut self.func_desc_map;
    }
}

