use eframe::egui;

pub struct CircleApp {
    head_pos: egui::Pos2,
    tail_pos: egui::Pos2,
    segment_length: f32,
    is_dragging: bool,
}

impl Default for CircleApp {
    fn default() -> Self {
        Self {
            head_pos: egui::Pos2::new(400.0, 300.0),
            tail_pos: egui::Pos2::new(350.0, 300.0),
            segment_length: 50.0,
            is_dragging: false,
        }
    }
}

impl eframe::App for CircleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create a canvas to draw on
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::drag(),
            );

            // Handle mouse interaction
            if response.dragged() {
                self.head_pos = response.hover_pos().unwrap_or(self.head_pos);
                self.is_dragging = true;
            } else {
                self.is_dragging = false;
            }

            // Update tail position to maintain fixed distance
            let direction = (self.head_pos - self.tail_pos).normalized();
            self.tail_pos = self.head_pos - direction * self.segment_length;

            // Draw the circles
            painter.circle_filled(
                self.head_pos,
                15.0,
                egui::Color32::from_rgb(200, 100, 100),  // Red for head
            );

            painter.circle_filled(
                self.tail_pos,
                10.0,
                egui::Color32::from_rgb(100, 200, 100),  // Green for tail
            );

            // Draw line connecting circles
            painter.line_segment(
                [self.head_pos, self.tail_pos],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 100)),
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