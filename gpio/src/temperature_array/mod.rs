use core::settings::Settings;
use core::traits::WorkWithHashMap;
use core::common_struct::NodeAddress;

extern crate rustc_serialize;
use rustc_serialize::json;

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct SensorNode{
    device_name: String,
    caption: String    
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct ResultNode{
    temperature: String,
    caption: String    
}

type FuncType = fn( &str ) -> String;

pub struct TemperatureArray {    
    sensor_vec: Vec<SensorNode>,
    nodes_with_temp_sensor: Vec<NodeAddress>
}

fn convert( input: String ) -> String {
    let mut final_temp : f32 = -100.0;
    for line_result in input.lines() {

        // find substring in string
        if line_result.contains( "t=" ) {
            println!( "line_result: {}", line_result );
            let sub_str = &line_result[ line_result.find( "t=" ).unwrap()+2 .. ];
            final_temp = match sub_str.parse::<i32>() {
                Ok( res ) => res as f32 / 1000 as f32,
                _ => -100.0
             };
        }
    }

    let final_string : String  = if final_temp > -100.0 {
        final_temp.to_string()
    }
    else {
        "не определена".to_string()
    };
    return final_string;
}

impl TemperatureArray {
    pub fn new() -> TemperatureArray {
        let temp_setting = Settings::new( "temperature_sensor.cfg" );
        let description = temp_setting.get( "temperature_sensor_array" );
        let sensor_vec_local: Vec<SensorNode> = match json::decode::<Vec::<SensorNode>>( &description ) {
            Ok( res ) => res,
            _ => panic!( "Can't decode params!" )
        };
        let another_nodes_with_temp_sensor = temp_setting.get( "another_nodes_with_temperature_sensor" );
        let nodes_with_temp_sensor_local: Vec<NodeAddress> = match json::decode::<Vec::<NodeAddress>>( &another_nodes_with_temp_sensor ) {
            Ok( res ) => res,
            _ => panic!( "Can't decode params!" )
        };
    
        return TemperatureArray{ sensor_vec: sensor_vec_local, nodes_with_temp_sensor: nodes_with_temp_sensor_local };
    }

    pub fn get( &self, func: FuncType ) -> String {
        let mut res_vec: Vec<ResultNode> = Vec::new();
        let len = self.sensor_vec.len();        
        for i in 0..len as usize {
            res_vec.push( ResultNode{ 
                temperature: convert( func( &self.sensor_vec[ i ].device_name ) ),
                caption: ( &self.sensor_vec[ i ].caption ).to_string()
            } );
        }
        return json::encode(&res_vec).unwrap();
    }

    pub fn get_other( &self ) -> String {
        let len = self.nodes_with_temp_sensor.len();        
        for i in 0..len as usize {
            // TODO: Need to add http async request for other nodes
        }
        return "".to_string();
    }
}