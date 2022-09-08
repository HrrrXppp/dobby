extern crate rustc_serialize;

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Params{
    pub desc: String,
    pub arg: String
}

impl Params {
    pub fn get_as_str( &self ) -> &str {
        if self.desc != "string" {
            panic!( "Get str from {}", self.desc );
        }
        return &self.arg;
    }
}

pub type RunFuncType = fn( &Vec< Params > ) -> String;

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct NodeAddress{
    address: String,
    port: u16    
}

