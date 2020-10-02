use nix::unistd::{fork, ForkResult, Pid};
use std::process;
use std::collections::{ HashMap };
use std::fs::{ File };
use std::io::{ BufReader,BufRead };
use nix::sys::{ socket, uio };
use nix::cmsg_space;
use std::os::unix::io::{ RawFd, FromRawFd };
use std::net::{ TcpStream };
use std::io::prelude::*;

pub trait Process{

    fn get_file( &self, file_name: &str ) -> String {
        return "Todo: get_file: ".to_string() + file_name;
    }

    fn process_message( &mut self, _message: &str ) -> ( String, String ) { 
        return ( self.get_file( "404.html" ), "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string() );
    }

    fn send_answer( &self, mut stream: TcpStream, result: &str, status_line: &str ){
    
        let response = format!("{}{}", status_line, result);
                                    
        stream.write( response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn run( &mut self ){

        let socket_name = process::id().to_string() + "queue";
        let socket = socket::socket( socket::AddressFamily::Unix,
                                     socket::SockType::Datagram,
                                     socket::SockFlag::empty(),
                                     None );

        match socket {
            Ok(socket) => {
               socket::bind( socket,
//                             &socket::SockAddr::Unix( socket::UnixAddr::new( &socket_name[..] ).unwrap() )
                             &socket::SockAddr::new_unix( &socket_name[..] ).unwrap()
                            ).unwrap();
                println!(  "Create socket {} {}", socket_name, socket );

                loop{
                    println!(  "Loop {}",  socket_name );
                    let mut buffer = vec![ 0u8; 512 ];
                    let message_vec = [uio::IoVec::from_mut_slice(&mut buffer)];
                    let mut cmsg_buffer = cmsg_space!( RawFd );
                    let result = socket::recvmsg( socket,
                                                  &message_vec,
                                                  Some( &mut cmsg_buffer ),
                                                  socket::MsgFlags::MSG_WAITALL );

                    match result {
                        Ok( res )=>{
                            let raw_fd = match res.cmsgs().next() {
                                Some( socket::ControlMessageOwned::ScmRights( raw_fd ) ) => raw_fd,
                                Some(_) => panic!("Unexpected control message"),
                                None => panic!("No control message")                            
                            };
                            let stream: TcpStream;
                            unsafe {
                                stream = TcpStream::from_raw_fd( raw_fd[ 0 ] );
                            }
                        
                            println!(  "result {}",  socket_name );
                            let message = String::from_utf8_lossy( message_vec[0].as_slice() );
                            println!( "{}", message );
                            let ( result, status ) = self.process_message( &message );
                            self.send_answer( stream, &result, &status );
                        },
                        Err( result ) =>{
                            println!("error in socket::recvmsg : {}", result );
                            process::exit( 0 );
                        }
                    }

                }
            },
            Err(socket) => println!("Unable to create socket for : {} {}", socket_name, socket)
        }
    }

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
