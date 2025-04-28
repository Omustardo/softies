use eframe::egui;
use crate::{creature::{Creature, Segment}, creature_ui::CreatureUI};

pub struct Snake {
    segments: Vec<Segment>,
    segment_length: f32,
    target_segments: usize,
    show_properties: bool,
    show_skin: bool,
    time: f32,
    center: egui::Pos2,
    direction: egui::Vec2,
    speed: f32,
    ui: CreatureUI,
}

impl Default for Snake {
    fn default() -> Self {
        let mut segments = Vec::new();
        let start_pos = egui::Pos2::new(400.0, 300.0);
        let mut current_pos = start_pos;
        let direction = egui::Vec2::new(1.0, 0.0);  // Start moving right

        // Create initial segments
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
            direction,
            speed: 100.0,
            ui: CreatureUI::new("snake"),
        }
    }
}

impl Creature for Snake {
    fn update_state(&mut self, ctx: &egui::Context) {
        // Update time
        self.time += ctx.input(|i| i.unstable_dt);

        // Update direction based on time (sinusoidal movement)
        let angle = self.time * 2.0;
        self.direction = egui::Vec2::new(angle.cos(), angle.sin()).normalized();

        // Update head position
        let delta = self.direction * self.speed * ctx.input(|i| i.unstable_dt);
        self.segments[0].pos = self.segments[0].pos + delta;

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
                self.direction
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

            // Draw the fill
            let mut fill_points = Vec::new();
            
            // Add left side points
            for segment in &self.segments {
                fill_points.push(segment.left_point);
            }
            
            // Add right side points in reverse order
            for segment in self.segments.iter().rev() {
                fill_points.push(segment.right_point);
            }

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
        "Snake"
    }
}

impl eframe::App for Snake {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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