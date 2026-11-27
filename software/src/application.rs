use eframe::egui;
use resvg::{tiny_skia, usvg};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::time::{Duration, Instant};

const DB: &str = "network.db3";
const FLOORS: [(&str, &str); 2] = [("0", "plan/floor_0.svg"), ("1", "plan/floor_1.svg")];

struct Location {
    id: i64,
    name: String,
    ssid: String,
}

struct Room {
    name: String,
    shape: tiny_skia::Path,
}

struct DataRow {
    frequency: String,
    rssi: i32,
    channel: i32,
    speed: f64,
}

struct RoomDetails {
    measurement_id: i64,
    date: String,
    time: String,
    rows: Vec<DataRow>,
}

// Everything loaded for one floor
struct Floor {
    texture: egui::TextureHandle,
    rooms: Vec<Room>,
    click_map: tiny_skia::Pixmap,
}

impl Floor {
    fn load(ctx: &egui::Context, path: &str) -> Floor {
        let (image, rooms) = load_svg(path);
        let [w, h] = image.size;
        let click_map = make_click_map(&rooms, w as u32, h as u32);
        let texture = ctx.load_texture(path, image, egui::TextureOptions::LINEAR);
        Floor { texture, rooms, click_map }
    }
}

struct App {
    connected: Arc<AtomicBool>,
    floors: [Option<Floor>; 2],
    selected_floor: usize,
    locations: Vec<Location>,
    selected_location: usize,
    scans: HashMap<String, i32>,   // room -> RSSI of its latest scan
    selected_room: Option<String>,
    details: Option<RoomDetails>,
    heatmap: Option<egui::TextureHandle>,
    heatmap_floor: Option<usize>,
    heatmap_stale: bool,
    last_refresh: Instant,
    refresh_location: Option<usize>,
}

pub fn run(connected: Arc<AtomicBool>) -> eframe::Result {
    let mut app = App::new(connected);

    eframe::run_ui_native("Wi-Fi Scanner", eframe::NativeOptions::default(), move |ui, _frame| {
        app.refresh();
        app.show_sidebar(ui);
        app.show_plan(ui);
    })
}

// Names of all rooms on all floors, sorted (sent to the ESP32 by the backend)
pub fn room_names() -> Vec<String> {
    let mut names = Vec::new();

    for (_, path) in FLOORS {
        if !std::path::Path::new(path).exists() {
            continue;
        }
        let mut rooms = Vec::new();
        collect_rooms(read_tree(path).root(), &mut rooms);
        names.extend(rooms.into_iter().map(|room| room.name));
    }

    names.sort();
    names.dedup();
    names
}

impl App {
    fn new(connected: Arc<AtomicBool>) -> App {
        App {
            connected,
            floors: [None, None],
            selected_floor: 0,
            locations: load_locations(),
            selected_location: 0,
            scans: HashMap::new(),
            selected_room: None,
            details: None,
            heatmap: None,
            heatmap_floor: None,
            heatmap_stale: true,
            last_refresh: Instant::now(),
            refresh_location: None,
        }
    }

    fn location_id(&self) -> Option<i64> {
        self.locations.get(self.selected_location).map(|l| l.id)
    }

    // Reloads data from the database: every 2 seconds, or right after the location changed
    fn refresh(&mut self) {
        if self.locations.is_empty() {
            if self.last_refresh.elapsed() > Duration::from_secs(1) {
                self.locations = load_locations();
                self.last_refresh = Instant::now();
            }
            return;
        }

        let location_changed = self.refresh_location != Some(self.selected_location);
        if !location_changed && self.last_refresh.elapsed() < Duration::from_secs(2) {
            return;
        }

        let Some(location_id) = self.location_id() else { return };

        let scans = load_scans(location_id);
        if scans != self.scans {
            self.scans = scans;
            self.heatmap_stale = true;
        }
        if let Some(room) = &self.selected_room {
            self.details = load_details(location_id, room);
        }

        self.refresh_location = Some(self.selected_location);
        self.last_refresh = Instant::now();
    }

    fn show_sidebar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("controls").show(ui, |ui| {
            ui.heading("Control Panel");

            ui.horizontal(|ui| {
                ui.label("Location:");
                let current = match self.locations.get(self.selected_location) {
                    Some(l) => format!("{} ({})", l.name, l.ssid),
                    None => "-".to_string(),
                };
                egui::ComboBox::from_id_salt("location")
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        for (i, l) in self.locations.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_location, i, format!("{} ({})", l.name, l.ssid));
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Floor:");
                egui::ComboBox::from_id_salt("floor")
                    .selected_text(FLOORS[self.selected_floor].0)
                    .show_ui(ui, |ui| {
                        for (i, (name, _)) in FLOORS.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_floor, i, *name);
                        }
                    });
            });

            ui.separator();
            self.show_details(ui);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                let status = if self.connected.load(Ordering::Relaxed) { "Connected" } else { "Not Connected" };
                ui.label(format!("Status: {status}"));
            });
            ui.ctx().request_repaint_after(Duration::from_millis(500));
        });
    }

    fn show_details(&self, ui: &mut egui::Ui) {
        match (&self.selected_room, &self.details) {
            (Some(room), Some(d)) => {
                ui.label(format!("Room: {room}"));
                ui.label(format!("Scan: #{}", d.measurement_id));
                ui.label(format!("Date: {}", d.date));
                ui.label(format!("Time: {}", d.time));
                for row in &d.rows {
                    ui.separator();
                    ui.label(format!("Frequency: {}", row.frequency));
                    ui.label(format!("RSSI: {} dBm", row.rssi));
                    ui.label(format!("Channel: {}", row.channel));
                    ui.label(format!("Speed: {}", row.speed));
                }
            }
            (Some(room), None) => {
                ui.label(format!("{room}: no scan yet"));
            }
            (None, _) => {
                ui.label("Click a room on the plan");
            }
        }
    }

    fn show_plan(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            let i = self.selected_floor;
            let location_id = self.location_id();

            if self.floors[i].is_none() {
                self.floors[i] = Some(Floor::load(ui.ctx(), FLOORS[i].1));
            }
            let floor = self.floors[i].as_ref().unwrap();

            if self.heatmap_stale || self.heatmap_floor != Some(i) {
                let image = make_heatmap(&floor.rooms, &self.scans, self.selected_room.as_deref(), floor.texture.size());
                self.heatmap = Some(ui.ctx().load_texture("heat", image, egui::TextureOptions::LINEAR));
                self.heatmap_floor = Some(i);
                self.heatmap_stale = false;
            }

            // Fit the plan into the free space, keeping its aspect ratio
            let size = floor.texture.size_vec2();
            let scale = (ui.available_width() / size.x).min(ui.available_height() / size.y);

            let response = ui.add(
                egui::Image::new(egui::load::SizedTexture::from_handle(&floor.texture))
                    .fit_to_exact_size(size * scale)
                    .sense(egui::Sense::click()),
            );

            if let Some(heatmap) = &self.heatmap {
                let whole = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                ui.painter().image(heatmap.id(), response.rect, whole, egui::Color32::WHITE);
            }

            if response.clicked() {
                if let Some(mouse) = response.interact_pointer_pos() {
                    let pos = (mouse - response.rect.min) / scale;   // pixel in the plan image
                    self.selected_room = room_at(floor, pos.x as u32, pos.y as u32);
                    self.details = match (location_id, &self.selected_room) {
                        (Some(id), Some(room)) => load_details(id, room),
                        _ => None,
                    };
                    self.heatmap_stale = true;
                    ui.ctx().request_repaint();
                }
            }
        });
    }
}

// ---------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------

fn solid_paint(r: u8, g: u8, b: u8, a: u8, anti_alias: bool) -> tiny_skia::Paint<'static> {
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(r, g, b, a);
    paint.anti_alias = anti_alias;
    paint
}

// Rooms with a scan are filled by signal strength, the selected room gets a black outline
fn make_heatmap(
    rooms: &[Room],
    scans: &HashMap<String, i32>,
    selected: Option<&str>,
    [w, h]: [usize; 2],
) -> egui::ColorImage {
    let mut pixmap = tiny_skia::Pixmap::new(w as u32, h as u32).unwrap();
    let no_transform = tiny_skia::Transform::identity();

    for room in rooms {
        if let Some(&rssi) = scans.get(&room.name) {
            let (r, g, b) = rssi_to_rgb(rssi);
            let paint = solid_paint(r, g, b, 150, true);
            pixmap.fill_path(&room.shape, &paint, tiny_skia::FillRule::Winding, no_transform, None);
        }
        if selected == Some(room.name.as_str()) {
            let paint = solid_paint(0, 0, 0, 255, true);
            let stroke = tiny_skia::Stroke { width: 2.0, ..Default::default() };
            pixmap.stroke_path(&room.shape, &paint, &stroke, no_transform, None);
        }
    }

    egui::ColorImage::from_rgba_premultiplied([w, h], pixmap.data())
}

// -90 dBm = red, -60 = yellow, -30 = green
fn rssi_to_rgb(rssi: i32) -> (u8, u8, u8) {
    let t = ((rssi + 90) as f32 / 60.0).clamp(0.0, 1.0);
    if t < 0.5 {
        (255, (510.0 * t) as u8, 0)
    } else {
        ((510.0 * (1.0 - t)) as u8, 255, 0)
    }
}

// Invisible image where room number n is filled with red value n (max 255 rooms).
// Reading the pixel under the mouse tells which room was clicked, in its exact shape.
fn make_click_map(rooms: &[Room], w: u32, h: u32) -> tiny_skia::Pixmap {
    let mut map = tiny_skia::Pixmap::new(w, h).unwrap();

    for (i, room) in rooms.iter().enumerate() {
        let paint = solid_paint((i + 1) as u8, 0, 0, 255, false);
        map.fill_path(&room.shape, &paint, tiny_skia::FillRule::Winding, tiny_skia::Transform::identity(), None);
    }

    map
}

fn room_at(floor: &Floor, x: u32, y: u32) -> Option<String> {
    let number = floor.click_map.pixel(x, y)?.red() as usize;
    let room = floor.rooms.get(number.checked_sub(1)?)?;   // 0 = no room
    Some(room.name.clone())
}

// ---------------------------------------------------------------
// Database
// ---------------------------------------------------------------

fn load_locations() -> Vec<Location> {
    query_locations().unwrap_or_default()
}

fn query_locations() -> rusqlite::Result<Vec<Location>> {
    let conn = Connection::open(DB)?;
    let mut stmt = conn.prepare("SELECT id, name, ssid FROM location ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(Location { id: row.get(0)?, name: row.get(1)?, ssid: row.get(2)? })
    })?;
    let list = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(list)
}

fn load_scans(location_id: i64) -> HashMap<String, i32> {
    query_scans(location_id).unwrap_or_default()
}

// Latest scan of every room (highest measurement id) and the best RSSI of its data rows
fn query_scans(location_id: i64) -> rusqlite::Result<HashMap<String, i32>> {
    let conn = Connection::open(DB)?;
    let mut stmt = conn.prepare(
        "SELECT m.room, MAX(d.rssi)
         FROM measurement m
         JOIN data d ON d.measurement_id = m.id
         WHERE m.id IN (SELECT MAX(id) FROM measurement WHERE location_id = ?1 GROUP BY room)
         GROUP BY m.room",
    )?;
    let rows = stmt.query_map([location_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)))?;
    let scans = rows.collect::<rusqlite::Result<HashMap<_, _>>>()?;
    Ok(scans)
}

fn load_details(location_id: i64, room: &str) -> Option<RoomDetails> {
    query_details(location_id, room).unwrap_or_else(|e| {
        println!("load_details: {e}");
        None
    })
}

fn query_details(location_id: i64, room: &str) -> rusqlite::Result<Option<RoomDetails>> {
    let conn = Connection::open(DB)?;

    let latest = conn
        .query_row(
            "SELECT id, date, time FROM measurement WHERE location_id = ?1 AND room = ?2 ORDER BY id DESC LIMIT 1",
            params![location_id, room],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        )
        .optional()?;
    let Some((measurement_id, date, time)) = latest else { return Ok(None) };

    let mut stmt = conn.prepare("SELECT frequency, rssi, channel, speed FROM data WHERE measurement_id = ?1")?;
    let rows = stmt.query_map([measurement_id], |row| {
        Ok(DataRow { frequency: row.get(0)?, rssi: row.get(1)?, channel: row.get(2)?, speed: row.get(3)? })
    })?;
    let rows = rows.collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(Some(RoomDetails { measurement_id, date, time, rows }))
}

// ---------------------------------------------------------------
// SVG
// ---------------------------------------------------------------

fn read_tree(path: &str) -> usvg::Tree {
    // The room shapes are invisible (no fill, transparent stroke) and usvg probably drops such
    // elements, so they get a barely visible fill
    let svg = std::fs::read_to_string(path).unwrap().replace(
        "fill:none;fill-opacity:1;stroke:#000000;stroke-opacity:0",
        "fill:#000000;fill-opacity:0.01;stroke:none",
    );
    usvg::Tree::from_str(&svg, &usvg::Options::default()).unwrap()
}

fn load_svg(path: &str) -> (egui::ColorImage, Vec<Room>) {
    let tree = read_tree(path);
    let size = tree.size().to_int_size();

    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height()).unwrap();
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let mut rooms = Vec::new();
    collect_rooms(tree.root(), &mut rooms);

    let image = egui::ColorImage::from_rgba_premultiplied(
        [size.width() as usize, size.height() as usize],
        pixmap.data(),
    );
    (image, rooms)
}

// Collects every named shape in the SVG, in image coordinates
fn collect_rooms(group: &usvg::Group, rooms: &mut Vec<Room>) {
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => collect_rooms(g, rooms),
            usvg::Node::Path(p) => {
                let name = p.id();
                if name.is_empty() || is_auto_name(name) {
                    continue;
                }
                if let Some(shape) = p.data().clone().transform(p.abs_transform()) {
                    rooms.push(Room { name: name.to_string(), shape });
                }
            }
            _ => {}
        }
    }
}

// Inkscape's generated ids like "rect30" or "path4" are not rooms
fn is_auto_name(id: &str) -> bool {
    ["path", "rect", "image", "text", "tspan", "g", "circle", "ellipse"]
        .iter()
        .filter_map(|prefix| id.strip_prefix(*prefix))
        .any(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
}