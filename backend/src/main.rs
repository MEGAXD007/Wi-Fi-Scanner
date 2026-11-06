mod backend;
mod application;

use std::thread;

fn main() {
    thread::spawn(|| backend::run());
    application::run();
}