use eframe::egui;

pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    let floor = ["0", "1"];
    let mut selected:usize = 0;

    eframe::run_ui_native("Wi-Fi Scanner", options, move |ui, _frame| {
        egui_extras::install_image_loaders(ui.ctx());
        egui::Panel::left("controls").show(ui, |ui| {
            ui.heading("Control Panel");

            egui::ComboBox::from_label("Floor")
            .selected_text(floor[selected])
            .show_ui(ui,|ui| {
                for (i, name) in floor.iter().enumerate() {
                    ui.selectable_value(&mut selected,i,*name);
                } 
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