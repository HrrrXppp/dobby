use crate::traits::{ Process, WorkWithHashMap, Parser };
use crate::settings::Settings;
use crate::router::Router;
use crate::executor::Executor;
use crate::file_cache::FileCache;

use std::process;
use nix::sys::{ socket, uio };
use nix::cmsg_space;
use std::os::unix::io::{ RawFd, FromRawFd };
use std::net::{ TcpStream };
use std::io::prelude::*;

pub struct Worker{
    file_name_404: String,
    route_rules: Router,
    executor: Executor,
    file_cache: FileCache
}

impl <'worker_lf> Worker{
    const ERROR_404_STATUS_LINE: &'worker_lf str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
    const OK_200_STATUS_LINE: &'worker_lf str = "HTTP/1.1 200 OK\r\n\r\n";

    pub fn new( file_settings_name: &str ) -> Worker {
        let new_settings = Settings::new( file_settings_name );
        return Worker{ 
                    route_rules: Router::new( &new_settings.get( "route_rules_file_name" ) ),
                    executor: Executor::new( &new_settings.get( "executor_rules_file_name" ) ), 
                    file_cache: FileCache::new( &new_settings.get( "file_cache_setting_file_name") ),
                    file_name_404: new_settings.get( "error_404_file_name" ) 
               };
    }

    fn process_message( &mut self, message: &str ) -> ( String, String ) {        
        println!(  "process_message {}", message );
        let caption = &message[..4];
        println!(  "Caption {}", caption );
        match caption {
           "GET " | "POST" => return self.process_get( &message[3..] ),
           "RUN " => {
               println!( "RUN" );
               return self.error_404();
           },
           _ => {
               println!( "something else!" );
               return self.error_404();           }
        }
    }

    fn process_get( &mut self, message: &str ) -> ( String, String ) {
        println!(  "process_get {}", message );

        let real_message = self.real_message_by_get( message );
        let func = real_message.0.to_lowercase();
        let args = real_message.1.to_lowercase();
        println!(  "    func, args {} {}", func, args );

        match self.route_rules.get_option( &func ){
            Some( action ) => {
                println!(  "    Action is {}", action );
                match self.executor.run( action, &args ){
                    Some( answer )  => {
                        println!( "    result in process_get is {}", answer );
                        return ( answer, Worker::OK_200_STATUS_LINE.to_string() );
                    },
                    None => return self.error_404()
                }
            },
            None => {
                println!(  "    Action is None" );
                return self.error_404();
            }
        }
    }

    fn error_404( &mut self ) -> ( String, String ){
        return ( self.file_cache.get_file( &self.file_name_404 ),
                 Worker::ERROR_404_STATUS_LINE.to_string() );
    } 

    fn send_answer( &self, mut stream: TcpStream, result: &str, status_line: &str ){
    
        let response = format!("{}{}", status_line, result);
                                    
        stream.write( response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

impl Parser for Worker{}

impl Process for Worker{    
    fn run( &mut self ){
        let socket_name = process::id().to_string() + "queue";
        let socket = socket::socket( socket::AddressFamily::Unix,
                                     socket::SockType::Datagram,
                                     socket::SockFlag::empty(),
                                     None );

        match socket {
            Ok(socket) => {
               socket::bind( socket,
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
}
