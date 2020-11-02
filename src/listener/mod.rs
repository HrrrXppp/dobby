use std::cmp::{min};
use std::convert::TryInto;
use std::net::{TcpStream, TcpListener};
use std::io::{Read};
use std::{thread, time, string};
use nix::unistd::{ Pid };
use nix::sys::{ socket, uio };
use std::os::unix::io::{ RawFd, AsRawFd };
extern crate num_cpus;
use crate::traits::Process;
use crate::worker::Worker;
use crate::settings::Settings;
use crate::traits::WorkWithHashMap;

pub struct Listener{
    worker_vec:  Vec< Pid >,
    socket_vec:  Vec< RawFd >,
    current_worker: usize,
    settings: Settings
}

impl Listener{

    pub fn new() -> Listener {
        return Listener{ worker_vec: Vec::new(), socket_vec: Vec::new(),
                         current_worker: 0, settings: Settings::new( "listener.cfg" )  }
    }

    pub fn create<T: Process >( &mut self, mut p: T )->Pid {
            return p.create();
    }

    fn handle_client( &mut self, mut stream: &TcpStream) {
        
        let mut buffer = [0; 512];
        stream.read(&mut buffer).unwrap();
        let request = string::String::from_utf8_lossy(&buffer[..]);
        
        if request.len() == 0{
            return;
        }

        println!("Request:\n {}", request );
        println!("Request len:\n {}", request.len() );

        let offset = request.find('\n').unwrap_or(request.len());
        let (first, _last) = request.split_at(offset);
        let message = &string::String::from_utf8_lossy( first.as_bytes() );

        let message_vec = uio::IoVec::from_slice( message.as_bytes() );

        match  socket::sendmsg( self.socket_vec[ self.current_worker ],
                         &[message_vec],
                         &[socket::ControlMessage::ScmRights  ( &[stream.as_raw_fd()] )],
                         socket::MsgFlags::empty(),
                         Option::None ) {
            Err( err ) => println!(  "Unable sendmsg socket {}: {}", self.socket_vec[ self.current_worker ], err ),
            Ok(_) => {}             
        }

        println!( "Send: {} {}", self.worker_vec[ self.current_worker ].to_string(), message );
        self.current_worker += 1;
        if self.socket_vec.len() == self.current_worker{
            self.current_worker = 0;
        }
    }
}

impl Process for Listener{
    fn run( &mut self ){
        println!(  "Run Listener" );

        let num_cpus: u16 = ((num_cpus::get() - 1) as u16).try_into().unwrap();

        if num_cpus < 2 {
            println!(  "Cpu core need more 2, you have {}", num_cpus + 1);
            return;
        }
        let mut max_worker_count: u16 = min( self.settings.get( "max_worker_count" ).parse().unwrap(),
                                         num_cpus );
        while max_worker_count != 0 {
            let worker = Worker::new( "worker.cfg" );
            let worker_pid = self.create(worker);
            self.worker_vec.push( worker_pid );
            let socket_name = worker_pid.to_string() + "queue";

            let socket = socket::socket( socket::AddressFamily::Unix,
                                         socket::SockType::Datagram,
                                         socket::SockFlag::empty(),
                                         None );
            match socket {
                Ok(socket) => {
                    self.socket_vec.push( socket );
                },
                Err(socket) => {
                    println!( "Unable to create socket for : {} {}", socket_name, socket )
                }
            }


            let ten_millis = time::Duration::from_millis(500);
            thread::sleep(ten_millis);
            println!(  "Open socket {}", socket_name );



            match socket::connect( self.socket_vec[ self.socket_vec.len() - 1 ],
                                   &socket::SockAddr::Unix( socket::UnixAddr::new( &socket_name[..] ).unwrap() ) ) {
                Ok(_) => println!(  "Connect socket {}", socket_name ),
                Err( err ) => println!(  "Unable connect socket {}: {}", socket_name, err )
            }

            max_worker_count -= 1;
        }

        let address = self.settings.get( "listening_address" );
        println!(  "listening_address:{}", address );
        let listener = TcpListener::bind( address );
        match listener {
            Ok(listener) => {
                println!("++++Ok(listener)++++");
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => self.handle_client(&stream),
                        Err(e) => println!("Unable to connect: {}", e)
                    }
                }
            },
            // TODO Make interrupt worker
            Err(e) => println!("Unable to bind: {}", e)
        }

    }
}
