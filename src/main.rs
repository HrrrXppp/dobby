extern crate num_cpus;
pub mod tests;
pub mod traits;
pub mod app;
pub mod listener;
pub mod worker;
pub mod settings;
pub mod router;
pub mod executor;
use crate::app::App;
use crate::listener::Listener;

fn main() {

let mut app = App{};
let listener = Listener::new();
app.create(listener);

  /*  app::Create::<listener::Listener, listener::Adapter<listener::Listener>>();

    CreateListener();
    CreateResponser();
    num_cp -= 2;
    while num_cpus != 0 {
        CreateWorker();
    }*/
}

/*
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let mut response = format!("{}{}", status_line, contents);


    let mut temp_file = File::open( "/sys/bus/w1/devices/28-00000a52d2e7/w1_slave" ).unwrap();
    let mut temp_string = String::new();
    temp_file.read_to_string( &mut temp_string ).unwrap(); 

    let mut final_temp : f32 = -100.0;

    for line_result in temp_string.lines() {

        // ищет подстроку в строке
        if line_result.contains( "t=" ) {
		
            let temp_int = line_result[ line_result.find( "t=" ).unwrap()+2 .. ].parse::<i32>().unwrap();
	    final_temp  = temp_int as f32 / 1000 as f32;
        }
    }

    let final_string : String  = if final_temp > -100.0 {
        final_temp.to_string()
    }
    else {
        "не определена".to_string()
    };
    
    response = str::replace( &response, "___temp___", &final_string );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
*/
