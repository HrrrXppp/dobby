use nix::unistd::{fork, ForkResult, Pid};
use std::collections::{ HashMap };
use std::fs::{ File };
use std::io::{ BufReader,BufRead };

pub trait Process{
    fn run( &mut self );

    fn create( &mut self ) -> Pid{
        println!(  "Process: Create>" );
        match fork() {
           Ok(ForkResult::Parent { child, .. }) => {
               println!("Continuing execution in parent process, new child has pid: {}", child);
               return child;
           },
           Ok(ForkResult::Child) => {
                println!("I'm a new child process");
                self.run();
                println!("End child process");
                return Pid::this();
           },
           Err(_) => println!("Fork failed"),
        }
        return Pid::from_raw( 0 );
    }
}


pub trait WorkWithHashMap{

    fn new( filename: &str ) ->Self;

    // TODO разобраться с этими функциями
    fn get_hash_map( &self ) -> &HashMap<String, String>;

    fn get_mut_hash_map( &mut self ) -> &mut HashMap<String, String>;

    fn load(  &mut self, filename: &str ) {
        let temp_file = File::open( filename ).unwrap();
        let reader = BufReader::new( temp_file );
        let hash_map = self.get_mut_hash_map();
        for line in reader.lines() {
            let line = line.unwrap();
            let offset = line.find('=').unwrap_or( line.len() );
            let (mut first, mut last) = line.split_at(offset);
            first = first.trim();
            last = &last[1..];
            last = last.trim();
            hash_map.insert( first.to_string(), last.to_string() );
        }
    }

    fn get( &self, setting_name : &str ) -> String  {
        let value = self.get_hash_map().get( setting_name );
        if None == value {
            panic!( "Unable get {} from settings", setting_name );
        }
        return String::from( value.unwrap() );
    }

    fn get_option( &self, setting_name : &String ) -> Option< &String >  {
        return self.get_hash_map().get( setting_name );
    }

}

pub trait Parser{

    fn real_message_by_get<'lifetime>( &self, message: &'lifetime str ) -> ( &'lifetime str, &'lifetime str ) {
        println!(  "real_message_by_get {}", message );
        let offset = message.find("HTTP").unwrap_or( message.len() );
        let (mut first, _last) = message.split_at(offset);
        println!(  "    first {}", first );
        if first.len() <= 1 {
            return ( "", "" );
        }
        first = first.trim();
        first = &first[1..];
        first = first.trim();
        let offset1 = first.find( "/" ).unwrap_or( first.len() );
        let ( func, mut args ) = first.split_at(offset1);
        if args.len() > 0 {
            args = &args[1..];
        }
        return ( func, args );
    }

}
