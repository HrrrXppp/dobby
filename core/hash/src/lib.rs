use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn get_hash( file_name: &str ) -> u64 {
    let mut hasher = DefaultHasher::new();
    file_name.hash(&mut hasher);
    return hasher.finish();
}

