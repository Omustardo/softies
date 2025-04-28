use eframe::egui;

pub mod snake;
pub mod ui;
pub use snake::Snake;
pub use ui::CreatureUI;

pub trait Creature {
    fn update_state(&mut self, ctx: &egui::Context);
    fn draw(&self, painter: &egui::Painter);
    fn get_segments(&self) -> &[Segment];
    fn get_segments_mut(&mut self) -> &mut [Segment];
    fn get_target_segments(&self) -> usize;
    fn set_target_segments(&mut self, count: usize);
    fn get_show_properties(&self) -> bool;
    fn set_show_properties(&mut self, show: bool);
    fn get_show_skin(&self) -> bool;
    fn set_show_skin(&mut self, show: bool);
    fn get_type_name(&self) -> &'static str;
}

#[derive(Clone)]
pub struct Segment {
    pos: egui::Pos2,
    radius: f32,
    color: egui::Color32,
    left_point: egui::Pos2,
    right_point: egui::Pos2,
}

impl Segment {
    fn new(pos: egui::Pos2, radius: f32, color: egui::Color32) -> Self {
        Self {
            pos,
            radius,
            color,
            left_point: pos,
            right_point: pos,
        }
    }

    fn update_side_points(&mut self, next_pos: Option<egui::Pos2>, prev_pos: Option<egui::Pos2>) {
        let direction = if let Some(next) = next_pos {
            (next - self.pos).normalized()
        } else if let Some(prev) = prev_pos {
            // For the last segment, use the same direction as the previous segment
            (self.pos - prev).normalized()
        } else {
            egui::Vec2::new(1.0, 0.0)  // Default direction if no segments
        };

        // Calculate perpendicular vector (90 degrees rotation)
        let perpendicular = egui::Vec2::new(-direction.y, direction.x);

        // Update side points
        self.left_point = self.pos + perpendicular * self.radius;
        self.right_point = self.pos - perpendicular * self.radius;
    }
}

pub struct SpiralCreature {
    segments: Vec<Segment>,
    segment_length: f32,
    target_segments: usize,
    show_properties: bool,
    show_skin: bool,
    time: f32,
    center: egui::Pos2,
    ui: CreatureUI,
}

impl Default for SpiralCreature {
    fn default() -> Self {
        let mut segments = Vec::new();
        let start_pos = egui::Pos2::new(400.0, 300.0);
        let mut current_pos = start_pos;
        let direction = egui::Vec2::new(-1.0, 0.0);  // Start moving left

        // Create initial segments with offset
        for i in 0..5 {
            segments.push(Segment::new(
                current_pos,
                if i == 0 { 15.0 } else { 10.0 },
                if i == 0 {
                    egui::Color32::from_rgb(200, 100, 100)  // Red for head
                } else {
                    egui::Color32::from_rgb(100, 200, 100)  // Green for body
                },
            ));
            current_pos = current_pos + direction * 50.0;  // Offset each segment
        }

        Self {
            segments,
            segment_length: 50.0,
            target_segments: 5,
            show_properties: false,
            show_skin: true,
            time: 0.0,
            center: egui::Pos2::new(400.0, 300.0),
            ui: CreatureUI::new("circle"),
        }
    }
}

impl Creature for SpiralCreature {
    fn update_state(&mut self, ctx: &egui::Context) {
        // Update time for spiral movement
        self.time += ctx.input(|i| i.unstable_dt);

        // Calculate spiral movement for head only
        let radius = 100.0 + self.time * 20.0;  // Increasing radius over time
        let angle = self.time * 2.0;  // Rotating angle
        let new_head_pos = egui::Pos2::new(
            self.center.x + radius * angle.cos(),
            self.center.y + radius * angle.sin(),
        );

        // Update head position
        self.segments[0].pos = new_head_pos;

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
            let new_pos = last_pos + direction * self.segment_length;
            self.segments.push(Segment::new(
                new_pos,
                10.0,
                egui::Color32::from_rgb(100, 200, 100),
            ));
        }
        while self.segments.len() > self.target_segments {
            self.segments.pop();
        }

        // Update side points for all segments
        for i in 0..self.segments.len() {
            let next_pos = if i < self.segments.len() - 1 {
                Some(self.segments[i + 1].pos)
            } else {
                None
            };
            let prev_pos = if i > 0 {
                Some(self.segments[i - 1].pos)
            } else {
                None
            };
            self.segments[i].update_side_points(next_pos, prev_pos);
        }
    }

    fn draw(&self, painter: &egui::Painter) {
        // Pre-allocate vectors for better performance
        let mut fill_points = Vec::with_capacity(self.segments.len() * 2);
        
        // Draw the skeleton first
        for segment in &self.segments {
            // Draw the main circle
            painter.circle_filled(
                segment.pos,
                segment.radius,
                segment.color,
            );

            // Draw side points
            painter.circle_filled(
                segment.left_point,
                3.0,
                egui::Color32::from_rgb(255, 255, 255),
            );
            painter.circle_filled(
                segment.right_point,
                3.0,
                egui::Color32::from_rgb(255, 255, 255),
            );
        }

        // Draw lines connecting segments
        for i in 0..self.segments.len() - 1 {
            painter.line_segment(
                [self.segments[i].pos, self.segments[i + 1].pos],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 100)),
            );
        }

        // Draw the skin on top if enabled
        if self.show_skin && self.segments.len() >= 2 {
            // Draw the left side
            for i in 0..self.segments.len() - 1 {
                painter.line_segment(
                    [self.segments[i].left_point, self.segments[i + 1].left_point],
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 160, 80)),
                );
            }

            // Draw the right side
            for i in 0..self.segments.len() - 1 {
                painter.line_segment(
                    [self.segments[i].right_point, self.segments[i + 1].right_point],
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 160, 80)),
                );
            }

            // Draw the ends
            painter.line_segment(
                [self.segments[0].left_point, self.segments[0].right_point],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 160, 80)),
            );
            painter.line_segment(
                [self.segments.last().unwrap().left_point, self.segments.last().unwrap().right_point],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 160, 80)),
            );

            // Build fill points more efficiently
            fill_points.clear();
            fill_points.extend(self.segments.iter().map(|s| s.left_point));
            fill_points.extend(self.segments.iter().rev().map(|s| s.right_point));

            // Draw the fill shape
            if fill_points.len() >= 3 {
                painter.add(egui::Shape::convex_polygon(
                    fill_points,
                    egui::Color32::from_rgba_premultiplied(100, 200, 100, 64),
                    egui::Stroke::NONE,
                ));
            }
        }
    }

    fn get_segments(&self) -> &[Segment] {
        &self.segments
    }

    fn get_segments_mut(&mut self) -> &mut [Segment] {
        &mut self.segments
    }

    fn get_target_segments(&self) -> usize {
        self.target_segments
    }

    fn set_target_segments(&mut self, count: usize) {
        self.target_segments = count;
    }

    fn get_show_properties(&self) -> bool {
        self.show_properties
    }

    fn set_show_properties(&mut self, show: bool) {
        self.show_properties = show;
    }

    fn get_show_skin(&self) -> bool {
        self.show_skin
    }

    fn set_show_skin(&mut self, show: bool) {
        self.show_skin = show;
    }

    fn get_type_name(&self) -> &'static str {
        "Circle"
    }
}

impl eframe::App for SpiralCreature {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repainting for smooth animation
        ctx.request_repaint();

        // UI controls in the top-left
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            self.ui.show_controls(ui, &mut self.target_segments, &mut self.show_properties, &mut self.show_skin);
        });

        // Properties panel
        if self.show_properties {
            egui::SidePanel::right("properties").show(ctx, |ui| {
                self.ui.show_properties(ui, &mut self.segments);
            });
        }

        // Main drawing area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create a canvas to draw on
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::drag(),
            );

            // Update center position if window is resized
            self.center = response.rect.center();

            // Update creature state
            self.update_state(ctx);

            // Draw the creature
            self.draw(&painter);
        });
    }
}

pub fn show_creature_controls(ui: &mut egui::Ui, creature: &mut dyn Creature, id: &str) {
    ui.horizontal(|ui| {
        if ui.button(format!("{}_add_segment", id)).clicked() {
            creature.set_target_segments((creature.get_target_segments() + 1).min(20));
        }
        if ui.button(format!("{}_remove_segment", id)).clicked() {
            creature.set_target_segments((creature.get_target_segments() - 1).max(2));
        }
        ui.add(egui::DragValue::new(&mut creature.get_target_segments())
            .speed(1)
            .clamp_range(2..=20)
            .prefix("Segments: "));
        
        ui.separator();
        
        if ui.button(format!("{}_toggle_properties", id)).clicked() {
            creature.set_show_properties(!creature.get_show_properties());
        }

        ui.separator();

        if ui.button(if creature.get_show_skin() { 
            format!("{}_hide_skin", id)
        } else { 
            format!("{}_show_skin", id)
        }).clicked() {
            creature.set_show_skin(!creature.get_show_skin());
        }
    });
}

pub fn show_creature_properties(ui: &mut egui::Ui, creature: &mut dyn Creature, id: &str) {
    ui.heading("Segment Properties");
    ui.separator();
    
    for (i, segment) in creature.get_segments_mut().iter_mut().enumerate() {
        ui.collapsing(format!("{}_segment_{}", id, i), |ui| {
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
            Box::new(|_cc| Box::new(SpiralCreature::default())),
        )
        .await
} 