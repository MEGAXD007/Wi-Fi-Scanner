use std::net::{TcpListener};
use rusqlite::{Connection, Result, params};
use std::io::{BufRead, BufReader, Write};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread;
use std::time::Duration;
use chrono::Local;

struct Data {
    ssid: String,
    room: String,
    multifrequency: bool,
    frequency: String,
    rssi: i32,
    channel: i32,
    speed: i32,
    date: String,
    time: String
}

pub fn run(connected: Arc<AtomicBool>, rooms: Arc<Vec<String>>) {
    open_db();

    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();      
    println!("Waiting for connection"); 

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let connected = Arc::clone(&connected);
        let rooms = Arc::clone(&rooms);

        thread::spawn(move || {
            stream.set_read_timeout(Some(Duration::from_secs(15))).unwrap();

            writeln!(&stream, "{}", rooms.join(";")).unwrap();

            connected.store(true, Ordering::Relaxed);

            for line in BufReader::new(stream).lines() {
                let Ok(line) = line else {break};
                if line == "ping" {
                    continue
                }          
                let data = decrypt(line.split(';').collect());
                write_db(&data);
                println!("Wrote to DB");
            }

            connected.store(false, Ordering::Relaxed)
        });
    }
}

fn decrypt(mdata: Vec<&str>) -> Data {
    let time = Local::now();

    Data {
        ssid: mdata[0].to_string(),
        room: mdata[1].to_string(),
        multifrequency: mdata[2] == "true",
        frequency: mdata[3].to_string(),
        rssi: mdata[4].parse::<i32>().unwrap(),
        channel: mdata[5].parse::<i32>().unwrap(),
        speed: mdata[6].parse::<i32>().unwrap(),
        date: time.format("%d.%m.%y").to_string(),
        time: time.format("%H:%M").to_string()
    }
}

fn open_db() -> Result<Connection> {
    let conn = Connection::open("network.db3")?;

    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;

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
                date            TEXT NOT NULL,
                time            TEXT NOT NULL,
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
        "INSERT INTO measurement (location_id, room, date, time) VALUES (?1, ?2, ?3, ?4)",
            params![1,data.room, data.date, data.time] // location_id currently hardcoded
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