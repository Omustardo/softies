use eframe::egui;

pub struct CircleApp {
    radius: f32,
}

impl Default for CircleApp {
    fn default() -> Self {
        Self { radius: 100.0 }
    }
}

impl eframe::App for CircleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Add a slider to control the circle's radius
            ui.add(egui::Slider::new(&mut self.radius, 10.0..=200.0).text("Radius"));

            // Create a canvas to draw on
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::drag(),
            );

            // Calculate the center of the canvas
            let center = response.rect.center();
            
            // Draw the circle
            painter.circle_filled(
                center,
                self.radius,
                egui::Color32::from_rgb(100, 150, 250),
            );
        });
    }
}

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    let web_options = eframe::WebOptions::default();
    
    eframe::WebRunner::new()
        .start(
            canvas_id,
            web_options,
            Box::new(|_cc| Box::new(CircleApp::default())),
        )
        .await
} 