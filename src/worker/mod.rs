use crate::traits::Process;
use crate::settings::Settings;
use crate::router::Router;
use crate::executor::Executor;
use crate::traits::WorkWithHashMap;

pub struct Worker{

    settings: Settings,
    route_rules: Router,
    executor: Executor
}

impl <'worker_lf> Worker{
    const ERROR_404_FILE_NAME: &'worker_lf str = "error_404_file_name";
    const ERROR_404_STATUS_LINE: &'worker_lf str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
    const OK_200_STATUS_LINE: &'worker_lf str = "HTTP/1.1 200 OK\r\n\r\n";

    pub fn new( file_settings_name: &str ) -> Worker {
        let new_settings = Settings::new( file_settings_name );
        let new_route_rules = Router::new( &new_settings.get( "route_rules_file_name" ) );
        let new_executor = Executor::new( &new_settings.get( "executor_rules_file_name" ) );
        return Worker{ settings: new_settings,
                       route_rules: new_route_rules,
                       executor: new_executor };
    }

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
        let ( first1, mut last1 ) = first.split_at(offset1);
        if last1.len() > 0 {
            last1 = &last1[1..];
        }
        return ( first1, last1 );
    }

    fn process_get( &mut self, message: &str ) -> ( String, String ) {
        println!(  "process_get {}", message );
        let real_message = self.real_message_by_get( message );
        let func = real_message.0.to_lowercase();
        let args = real_message.1.to_lowercase();
        println!(  "    func, args {} {}", func, args );
        let action = self.route_rules.get_option( &func );
        if None == action {
            println!(  "    Action is None" );
            return ( self.get_file( &self.settings.get( &Worker::ERROR_404_FILE_NAME ) ),
                     Worker::ERROR_404_STATUS_LINE.to_string() );
        }
        println!(  "    Action is {}", action.unwrap() );
        let result = self.executor.run( action.unwrap(), &args );
        if None == result {
            return ( self.get_file( &self.settings.get( &Worker::ERROR_404_FILE_NAME ) ),
                     Worker::ERROR_404_STATUS_LINE.to_string() );
        } 
        let answer = result.unwrap();
        println!( "    result in process_get is {}", answer );
        return ( answer, Worker::OK_200_STATUS_LINE.to_string() );
    }

    fn error_404( &self ) -> ( String, String ){
        return ( self.get_file( &self.settings.get( &Worker::ERROR_404_FILE_NAME ) ),
        Worker::ERROR_404_STATUS_LINE.to_string() );
    } 
}

impl Process for Worker{
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
}
