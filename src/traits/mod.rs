use nix::unistd::{fork, ForkResult, Pid};

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
