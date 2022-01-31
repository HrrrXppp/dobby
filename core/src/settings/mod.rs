use std::collections::{ HashMap };
use crate::traits::WorkWithHashMap;

pub struct Settings{
    settings: HashMap<String, String>
}

impl Settings{
}

impl WorkWithHashMap for Settings{

    fn new( filename: &str ) -> Settings {
        let mut new_settings = Settings{ settings: HashMap::new() };
        new_settings.load( filename );
        return new_settings;
    }

    fn get_hash_map( &self ) -> &HashMap<String, String> {
        return &self.settings;
    }

    fn get_mut_hash_map( &mut self ) -> &mut HashMap<String, String> {
        return &mut self.settings;
    }

}

