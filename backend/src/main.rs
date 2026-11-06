use std::io::{Read};
use std::net::{TcpListener, TcpStream};
use std::thread;
use rusqlite::{Connection, Result, params};
use std::io::{BufRead, BufReader};

struct Data {
    room: String,
    multifrequency: bool,
    timestemp: i32,
    frequency: String,
    rssi: i32,
    channel: i32,
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
        multifrequency: mdata[1] == "true",
        timestemp: 1900, //set time by backend
        frequency: mdata[2].to_string(),
        rssi: mdata[3].parse::<i32>().unwrap(),
        channel: mdata[4].parse::<i32>().unwrap(),
        speed: mdata[5].parse::<i32>().unwrap()
    }
}

fn open_db() -> Result<Connection> {
    let conn = Connection::open("network.db3")?;

    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    conn.execute(
    "CREATE TABLE IF NOT EXISTS location (
                id              INTEGER PRIMARY KEY,
                name            TEXT NOT NULL,
                ssid            TEXT NOT NULL
            ) STRICT",
            (),
        )?;

    conn.execute(
    "CREATE TABLE IF NOT EXISTS measurement (
                id              INTEGER PRIMARY KEY,
                location_id     INTEGER NOT NULL,
                room            TEXT NOT NULL,
                ts              INTEGER NOT NULL,
                FOREIGN KEY (location_id) REFERENCES location(id)
            ) STRICT",
            (),
        )?;

    conn.execute(
    "CREATE TABLE IF NOT EXISTS data (
                id              INTEGER PRIMARY KEY,
                measurement_id  INTEGER NOT NULL,
                frequency       TEXT NOT NULL,
                rssi            INTEGER NOT NULL,
                channel         INTEGER NOT NULL,
                speed           REAL NOT NULL,
                FOREIGN KEY (measurement_id) REFERENCES measurement(id)
            ) STRICT",
            (),
        )?;

    Ok(conn)
}

fn write_db(data: &Data) -> Result<Connection> {
    let conn = Connection::open("network.db3")?;

    if !data.multifrequency {
        conn.execute(
        "INSERT INTO measurement (location_id, room, ts) VALUES (?1, ?2, ?3)",
            params![1,data.room, data.timestemp]
            )?;
        }

    let id: i64 = conn.query_row(
        "SELECT id FROM measurement ORDER BY id DESC LIMIT 1",
        (),
        |row| row.get(0),
    )?;

    conn.execute(
    "INSERT INTO data (measurement_id, frequency, rssi, channel, speed) VALUES (?1, ?2, ?3, ?4, ?5)", //Assign same id for same scan with different frequencys needed
        params![id, data.frequency, data.rssi, data.channel, data.speed]
        )?;

    Ok(conn)
}