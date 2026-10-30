use std::io::{Read};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let peer = stream.peer_addr().unwrap();
    println!("Connected to: {}", peer);

    let mut buffer = [0u8; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Disconnected: {}", peer);
                break;
            }
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buffer[..n]);
                println!("Recieved from {}: {}", peer, msg);
                let daten: Vec<&str> = msg.split(';').collect();
                println!("{:?}", daten);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server available on: 127.0.0.1:7878");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => eprintln!("Connection error: {}", e),
        }
    }
    Ok(())
}