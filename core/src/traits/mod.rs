use std::collections::{ HashMap };
use std::fs::{ File };
use std::io::{ BufReader,BufRead };
use nix::unistd::{fork, ForkResult, Pid};

pub trait Process{
    fn run( &mut self );

    fn init( &mut self );

    fn create( &mut self ) -> Pid{
        println!(  "Process: Create>" );
        match unsafe{ fork() } {
           Ok(ForkResult::Parent { child, .. }) => {
               println!("Continuing execution in parent process, new child has pid: {}", child);
               return child;
           },
           Ok(ForkResult::Child) => {
                println!("I'm a new child process");
                self.init();
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
