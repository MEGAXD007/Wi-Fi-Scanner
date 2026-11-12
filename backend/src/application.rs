use eframe::egui;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::time::Duration;

pub fn run(connected: Arc<AtomicBool>) -> eframe::Result {
    let options = eframe::NativeOptions::default();

    let floor = ["0", "1"];
    let mut selected:usize = 0;

    eframe::run_ui_native("Wi-Fi Scanner", options, move |ui, _frame| {
        egui_extras::install_image_loaders(ui.ctx());
        
        egui::Panel::left("controls").show(ui, |ui| {
            ui.heading("Control Panel");

            ui.horizontal(|ui| {
                ui.label("Floor:");
                egui::ComboBox::from_id_salt("floor")
                .selected_text(floor[selected])
                .show_ui(ui,|ui| {
                    for (i, name) in floor.iter().enumerate() {
                        ui.selectable_value(&mut selected, i, *name);
                    } 
                });
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                if connected.load(Ordering::Relaxed) {
                    ui.label("Status: Connected");
                } else {
                    ui.label("Status: Not Connected");
                }
                ui.ctx().request_repaint_after(Duration::from_millis(500));
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            let plan = match selected {
                0 => egui::include_image!("../plan/floor_0.jpg"),
                _ => egui::include_image!("../plan/floor_1.jpg"),
            };
            ui.image(plan)
        });
    })
}