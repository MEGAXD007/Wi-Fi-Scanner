use std::sync::{atomic::AtomicBool, Arc};
use std::thread;

mod backend;
mod application;

fn main() {
    let connected = Arc::new(AtomicBool::new(false));

    let connected_backend = Arc::clone(&connected);
    thread::spawn(move || backend::run(connected_backend));

    application::run(connected).unwrap();
}