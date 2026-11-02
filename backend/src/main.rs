use std::io::{Read};
use std::net::{TcpListener, TcpStream};
use std::thread;
use rusqlite::{Connection, Result, params};
///////////////////////////////////////////
// mdata includesn the measured data.
// mdata[0]: room id
// mdata[1]: timestamp
// mdata[3]: speed
//
// Object to change
///////////////////////////////////////////

fn main() -> std::io::Result<()> {
    open_db();

    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Waiting for connection");

    loop {
        let (mut stream, addr) = listener.accept()?;
        println!("Connected to: {}", addr);

        let mut buffer = String::new();
        stream.read_to_string(&mut buffer)?;
        let mdata: Vec<&str> = buffer.split(';').collect();
        println!("{:?}", mdata);
        write_db(mdata);
    }
}

fn open_db() -> Result<Connection> {
    let conn = Connection::open("network.db3")?;

    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS measurement (
            id   INTEGER PRIMARY KEY,
            room TEXT NOT NULL,
            ts   INTEGER NOT NULL
        ) STRICT",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS data (
            id             INTEGER PRIMARY KEY,
            measurement_id INTEGER NOT NULL,
            speed          REAL NOT NULL,
            FOREIGN KEY (measurement_id) REFERENCES measurement(id)
        ) STRICT",
        (),
    )?;

    Ok(conn)
}

fn write_db(mdata: Vec<&str>) -> Result<Connection> {
    let conn = Connection::open("network.db3")?;

    conn.execute(
            "INSERT INTO measurement (room, ts) VALUES (?1, ?2)",
            params![mdata[0], mdata[1]]
    )?;

    let id = conn.last_insert_rowid();


    conn.execute(
            "INSERT INTO data (measurement_id, speed) VALUES (?1, ?2)",
            params![id, mdata[2]]
    )?;

    Ok(conn)
}
