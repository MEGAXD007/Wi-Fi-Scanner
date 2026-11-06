use std::io::{Read};
use std::net::{TcpListener, TcpStream};
use std::thread;
use rusqlite::{Connection, Result, params};
use std::io::{BufRead, BufReader};

struct Data {
    room: String,
    multi_frequency: bool,
    timestemp: i32,
    frequency: String,
    quality: String,
    speed: i32
}

fn main() {
    open_db();

    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    println!("Waiting for connection");

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        for line in BufReader::new(stream).lines() {
            let line = line.unwrap();
            let data = decrypt(line.split(';').collect());
            write_db(&data);
            println!("Wrote to DB")
        }
    }
}

fn decrypt(mdata: Vec<&str>) -> Data {
    Data {
        room: mdata[0].to_string(),
        multi_frequency: mdata[1] == "true",
        timestemp: mdata[2].parse::<i32>().unwrap(),
        frequency: mdata[3].to_string(),
        quality: mdata[4].to_string(),
        speed: mdata[5].parse::<i32>().unwrap()
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
                id              INTEGER PRIMARY KEY,
                measurement_id  INTEGER NOT NULL,
                frequency       TEXT NOT NULL,
                quality         TEXT NOT NULL,
                speed           REAL NOT NULL,
                FOREIGN KEY (measurement_id) REFERENCES measurement(id)
            ) STRICT",
            (),
        )?;

    Ok(conn)
}

fn write_db(data: &Data) -> Result<Connection> {
    let conn = Connection::open("network.db3")?;

    if !data.multi_frequency {
        conn.execute(
        "INSERT INTO measurement (room, ts) VALUES (?1, ?2)",
            params![data.room, data.timestemp]
            )?;
        }

    let id: i64 = conn.query_row(
        "SELECT id FROM measurement ORDER BY id DESC LIMIT 1",
        (),
        |row| row.get(0),
    )?;

    conn.execute(
    "INSERT INTO data (measurement_id, frequency, quality, speed) VALUES (?1, ?2, ?3, ?4)", //Assign same id for same scan with different frequencys needed
        params![id, data.frequency, data.quality, data.speed]
        )?;

    Ok(conn)
}