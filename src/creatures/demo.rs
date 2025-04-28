use eframe::egui;
use crate::{creature::{Creature, Segment}, creature_ui::CreatureUI};
use rand::Rng;

pub struct DemoCreature {
    segments: Vec<Segment>,
    segment_length: f32,
    target_segments: usize,
    show_properties: bool,
    show_skin: bool,
    time: f32,
    center: egui::Pos2,
    ui: CreatureUI,
    target_pos: egui::Pos2,
    speed: f32,
}

impl Default for DemoCreature {
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
            ui: CreatureUI::new("demo"),
            target_pos: start_pos,
            speed: 200.0,
        }
    }
}

impl Creature for DemoCreature {
    fn update_state(&mut self, ctx: &egui::Context) {
        // Only update time if we're actually moving
        let dt = ctx.input(|i| i.unstable_dt);
        if dt > 0.0 {
            self.time += dt;

            // Get mouse position or generate random target
            let mouse_pos = ctx.input(|i| i.pointer.hover_pos());
            if let Some(pos) = mouse_pos {
                self.target_pos = pos;
            } else {
                // If no mouse position, check if we need a new random target
                let head_pos = self.segments[0].pos;
                let distance_to_target = (head_pos - self.target_pos).length();
                if distance_to_target < 10.0 {
                    // Generate new random target within screen bounds
                    let mut rng = rand::thread_rng();
                    let screen_rect = ctx.screen_rect();
                    self.target_pos = egui::Pos2::new(
                        rng.gen_range(screen_rect.min.x..screen_rect.max.x),
                        rng.gen_range(screen_rect.min.y..screen_rect.max.y),
                    );
                }
            }

            // Calculate direction to target
            let head_pos = self.segments[0].pos;
            let direction = (self.target_pos - head_pos).normalized();
            
            // Move head toward target
            let new_head_pos = head_pos + direction * self.speed * dt;

            // Only update if position actually changed
            if new_head_pos != head_pos {
                // Update head position
                self.segments[0].pos = new_head_pos;

                // Update segment positions to maintain fixed distances
                for i in 1..self.segments.len() {
                    let direction = (self.segments[i-1].pos - self.segments[i].pos).normalized();
                    self.segments[i].pos = self.segments[i-1].pos - direction * self.segment_length;
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
    }

    fn draw(&self, painter: &egui::Painter) {
        // Pre-allocate vectors for better performance
        let mut shapes = Vec::with_capacity(self.segments.len() * 2); // Reduced capacity since we'll combine shapes
        
        // Draw the skeleton first
        for segment in &self.segments {
            // Add main circle
            shapes.push(egui::Shape::circle_filled(
                segment.pos,
                segment.radius,
                segment.color,
            ));
        }

        // Draw all connecting lines in a single shape
        if self.segments.len() > 1 {
            let mut points = Vec::with_capacity(self.segments.len());
            for segment in &self.segments {
                points.push(segment.pos);
            }
            shapes.push(egui::Shape::line(
                points,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 100)),
            ));
        }

        // Draw the skin if enabled
        if self.show_skin && self.segments.len() >= 2 {
            // Create fill polygons between adjacent segments
            for i in 0..self.segments.len() - 1 {
                let mut segment_points = Vec::with_capacity(4);
                segment_points.push(self.segments[i].left_point);
                segment_points.push(self.segments[i].right_point);
                segment_points.push(self.segments[i + 1].right_point);
                segment_points.push(self.segments[i + 1].left_point);
                
                shapes.push(egui::Shape::convex_polygon(
                    segment_points,
                    egui::Color32::from_rgba_premultiplied(100, 200, 100, 64),
                    egui::Stroke::NONE,
                ));
            }

            // Draw all side lines in a single shape
            let mut side_points = Vec::with_capacity(self.segments.len() * 2);
            // Add left side points from head to tail
            for segment in &self.segments {
                side_points.push(segment.left_point);
            }
            // Add right side points from tail to head
            for segment in self.segments.iter().rev() {
                side_points.push(segment.right_point);
            }
            shapes.push(egui::Shape::line(
                side_points,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 160, 80)),
            ));
        }

        // Draw all shapes in a single batch
        painter.extend(shapes);
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
        "Demo"
    }
}

impl eframe::App for DemoCreature {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint(); // critical for smooth animation!

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
            if response.rect.center() != self.center {
                self.center = response.rect.center();
                ctx.request_repaint();
            }

            // Only update state if we're visible and not paused
            if response.rect.width() > 0.0 && response.rect.height() > 0.0 {
                self.update_state(ctx);
            }

            // Draw the creature
            self.draw(&painter);
        });
    }
} 