use core::traits::Process;
use core::traits::WorkWithHashMap;
use core::file_cache::FileCache;
use core::settings::Settings;
use nix::unistd::{ Pid };

pub struct App{
}

impl App{
    pub fn create<T: Process >( &mut self, mut p : T )->Pid {
            return p.create();
    }

    pub fn init() {
        let worker_settings = Settings::new( "worker.cfg" );
        let file_cache_settings = Settings::new( &worker_settings.get( "file_cache_setting_file_name") );        
        FileCache::reset( &file_cache_settings.get( "file_cache_folder") );
    }
}
