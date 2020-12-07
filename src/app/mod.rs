use crate::traits::Process;
use nix::unistd::{ Pid };

pub struct App{
}

impl App{
    pub fn init( &self) {
        let service_directory_name = "./temp";
    }

    pub fn create<T: Process >( &mut self, mut p : T )->Pid {
            return p.create();
    }
}
