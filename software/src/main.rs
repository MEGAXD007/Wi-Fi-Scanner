use std::sync::{atomic::AtomicBool, Arc};
use std::thread;
use std::fs::File;
use std::io::prelude::*;

mod backend;
mod application;

fn main() {
    let connected = Arc::new(AtomicBool::new(false));
    let rooms = Arc::new(application::room_names());

    let connected_backend = Arc::clone(&connected);
    let rooms_backend = Arc::clone(&rooms);

    thread::spawn(move || backend::run(connected_backend, rooms_backend));

    application::run(connected).unwrap();
}