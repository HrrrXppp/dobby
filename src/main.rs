extern crate num_cpus;
pub mod traits;
pub mod app;
pub mod listener;
pub mod worker;
pub mod router;
pub mod executor;
use crate::app::App;
use crate::listener::Listener;

fn main() {
    App::init();
    let mut app = App{};
    let listener = Listener::new();
    app.create(listener);
}