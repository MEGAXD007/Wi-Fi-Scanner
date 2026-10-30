use std::io::{Read};
use std::net::{TcpListener, TcpStream};
use std::thread;
use rusqlite::{Connection, Result};

fn main() -> std::io::Result<()> {
    thread::spawn(|| open_db());

    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Waiting for connection");

    loop {
        let (mut stream, addr) = listener.accept()?;
        println!("Connected to: {}", addr);

        let mut buffer = String::new();
        stream.read_to_string(&mut buffer)?;
        let daten: Vec<&str> = buffer.split(';').collect();
        println!("{:?}", daten);
    }
}

fn open_db() {
    let conn = Connection::open("network.db");
}