use eframe::egui;

pub struct CircleApp {
    segments: Vec<egui::Pos2>,
    segment_length: f32,
    is_dragging: bool,
    target_segments: usize,
}

impl Default for CircleApp {
    fn default() -> Self {
        Self {
            segments: vec![
                egui::Pos2::new(400.0, 300.0),  // Head
                egui::Pos2::new(350.0, 300.0),  // First segment
            ],
            segment_length: 50.0,
            is_dragging: false,
            target_segments: 2,
        }
    }
}

impl eframe::App for CircleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // UI controls in the top-left
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Add Segment").clicked() {
                    self.target_segments = (self.target_segments + 1).min(20);
                }
                if ui.button("Remove Segment").clicked() {
                    self.target_segments = (self.target_segments - 1).max(2);
                }
                ui.add(egui::DragValue::new(&mut self.target_segments)
                    .speed(1)
                    .clamp_range(2..=20)
                    .prefix("Segments: "));
            });
        });

        // Main drawing area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create a canvas to draw on
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::drag(),
            );

            // Handle mouse interaction
            if response.dragged() {
                self.segments[0] = response.hover_pos().unwrap_or(self.segments[0]);
                self.is_dragging = true;
            } else {
                self.is_dragging = false;
            }

            // Update segment positions to maintain fixed distances
            for i in 1..self.segments.len() {
                let direction = (self.segments[i-1] - self.segments[i]).normalized();
                self.segments[i] = self.segments[i-1] - direction * self.segment_length;
            }

            // Adjust number of segments if needed
            while self.segments.len() < self.target_segments {
                let last_pos = *self.segments.last().unwrap();
                let direction = if self.segments.len() > 1 {
                    (self.segments[self.segments.len()-2] - last_pos).normalized()
                } else {
                    egui::Vec2::new(-1.0, 0.0)
                };
                self.segments.push(last_pos + direction * self.segment_length);
            }
            while self.segments.len() > self.target_segments {
                self.segments.pop();
            }

            // Draw the segments
            for (i, segment) in self.segments.iter().enumerate() {
                let color = if i == 0 {
                    egui::Color32::from_rgb(200, 100, 100)  // Red for head
                } else {
                    egui::Color32::from_rgb(100, 200, 100)  // Green for body
                };
                
                let radius = if i == 0 { 15.0 } else { 10.0 };
                painter.circle_filled(*segment, radius, color);
            }

            // Draw lines connecting segments
            for i in 0..self.segments.len() - 1 {
                painter.line_segment(
                    [self.segments[i], self.segments[i + 1]],
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 100)),
                );
            }
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