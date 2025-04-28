use eframe::egui;

#[derive(Clone)]
pub struct Segment {
    pos: egui::Pos2,
    radius: f32,
    color: egui::Color32,
}

pub struct CircleApp {
    segments: Vec<Segment>,
    segment_length: f32,
    is_dragging: bool,
    target_segments: usize,
    show_properties: bool,
}

impl Default for CircleApp {
    fn default() -> Self {
        Self {
            segments: vec![
                Segment {
                    pos: egui::Pos2::new(400.0, 300.0),
                    radius: 15.0,
                    color: egui::Color32::from_rgb(200, 100, 100),
                },
                Segment {
                    pos: egui::Pos2::new(350.0, 300.0),
                    radius: 10.0,
                    color: egui::Color32::from_rgb(100, 200, 100),
                },
            ],
            segment_length: 50.0,
            is_dragging: false,
            target_segments: 2,
            show_properties: false,
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
                
                ui.separator();
                
                if ui.button("Toggle Properties").clicked() {
                    self.show_properties = !self.show_properties;
                }
            });
        });

        // Properties panel
        if self.show_properties {
            egui::SidePanel::right("properties").show(ctx, |ui| {
                ui.heading("Segment Properties");
                ui.separator();
                
                for (i, segment) in self.segments.iter_mut().enumerate() {
                    ui.collapsing(format!("Segment {}", i), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Radius:");
                            ui.add(egui::DragValue::new(&mut segment.radius)
                                .speed(0.5)
                                .clamp_range(5.0..=30.0));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            let mut color = [
                                segment.color.r(),
                                segment.color.g(),
                                segment.color.b(),
                            ];
                            if ui.color_edit_button_srgb(&mut color).changed() {
                                segment.color = egui::Color32::from_rgb(
                                    color[0],
                                    color[1],
                                    color[2],
                                );
                            }
                        });
                    });
                }
            });
        }

        // Main drawing area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create a canvas to draw on
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::drag(),
            );

            // Handle mouse interaction
            if response.dragged() {
                self.segments[0].pos = response.hover_pos().unwrap_or(self.segments[0].pos);
                self.is_dragging = true;
            } else {
                self.is_dragging = false;
            }

            // Update segment positions to maintain fixed distances
            for i in 1..self.segments.len() {
                let direction = (self.segments[i-1].pos - self.segments[i].pos).normalized();
                self.segments[i].pos = self.segments[i-1].pos - direction * self.segment_length;
            }

            // Adjust number of segments if needed
            while self.segments.len() < self.target_segments {
                let last_pos = self.segments.last().unwrap().pos;
                let direction = if self.segments.len() > 1 {
                    (self.segments[self.segments.len()-2].pos - last_pos).normalized()
                } else {
                    egui::Vec2::new(-1.0, 0.0)
                };
                self.segments.push(Segment {
                    pos: last_pos + direction * self.segment_length,
                    radius: 10.0,
                    color: egui::Color32::from_rgb(100, 200, 100),
                });
            }
            while self.segments.len() > self.target_segments {
                self.segments.pop();
            }

            // Draw the segments
            for segment in &self.segments {
                painter.circle_filled(
                    segment.pos,
                    segment.radius,
                    segment.color,
                );
            }

            // Draw lines connecting segments
            for i in 0..self.segments.len() - 1 {
                painter.line_segment(
                    [self.segments[i].pos, self.segments[i + 1].pos],
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