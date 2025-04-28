use eframe::egui;
use softies::CircleApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Circle Demo",
        options,
        Box::new(|_cc| Box::new(CircleApp::default())),
    )
}
