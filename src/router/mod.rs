use std::collections::{ HashMap };
use core::traits::WorkWithHashMap;

pub struct Router{
    route_rules:  HashMap<String, String>
}

impl Router{
}

impl WorkWithHashMap for Router{
    fn new( filename: &str ) ->Router {
        let mut new_route_rules = Router{ route_rules: HashMap::new() };
        new_route_rules.load( filename );
        return new_route_rules;
    }

    fn get_hash_map( &self ) -> &HashMap<String, String>{
        return &self.route_rules;
    }

    fn get_mut_hash_map( &mut self ) -> &mut HashMap<String, String> {
        return &mut self.route_rules;
    }

}
