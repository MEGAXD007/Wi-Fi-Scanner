use eframe::egui;

pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    let mut name = String::from("Arthur");
    let mut age = 42;

    eframe::run_ui_native("Wi-Fi Scanner", options, move |ui, _frame| {
        egui_extras::install_image_loaders(ui.ctx());

        ui.heading("My egui Application");
        ui.horizontal(|ui| {
            ui.label("Your name: ");
            ui.text_edit_singleline(&mut name);
        });
        ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
        if ui.button("Increment").clicked() {
            age += 1;
        }
        ui.label(format!("Hello '{name}', age {age}"));
    })
}