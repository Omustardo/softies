use eframe::egui;
use rapier2d::prelude::*;
use crate::creature::{Creature, Segment, PhysicsWorld};
use std::any::Any;

const PIXELS_PER_METER: f32 = 50.0;

pub struct TestChain {
    segments: Vec<Segment>,
    target_segments: usize,
    show_properties: bool,
    show_skin: bool,
    physics_world: PhysicsWorld,
    rigid_body_handles: Vec<RigidBodyHandle>,
    joint_handles: Vec<ImpulseJointHandle>,
    time: f32,
    startup_delay: f32,
}

impl Default for TestChain {
    fn default() -> Self {
        let mut segments = Vec::new();
        let start_pos = egui::Pos2::new(400.0, 300.0);
        let mut current_pos = start_pos;

        // Create 10 segments
        for i in 0..10 {
            segments.push(Segment::new(
                current_pos,
                if i == 0 { 15.0 } else { 10.0 },  // Larger head
                if i == 0 {
                    egui::Color32::from_rgb(200, 100, 100)  // Red for head
                } else {
                    egui::Color32::from_rgb(100, 200, 100)  // Green for body
                },
            ));
            current_pos = current_pos + egui::Vec2::new(30.0, 0.0);  // 30 pixels spacing
        }

        let mut physics_world = PhysicsWorld::default();
        let mut rigid_body_handles = Vec::new();
        let mut joint_handles = Vec::new();

        // Create physics bodies
        for segment in &segments {
            // Convert from pixels to meters for physics
            let pos_meters = vector![
                segment.pos.x / PIXELS_PER_METER,
                segment.pos.y / PIXELS_PER_METER
            ];
            let radius_meters = segment.radius / PIXELS_PER_METER;

            let rigid_body = RigidBodyBuilder::dynamic()
                .translation(pos_meters)
                .linear_damping(0.5)  // Increased damping for stability
                .angular_damping(0.5)
                .build();
            
            let handle = physics_world.rigid_body_set.insert(rigid_body);
            rigid_body_handles.push(handle);

            // Create collider
            let collider = ColliderBuilder::ball(radius_meters)
                .restitution(0.1)
                .friction(0.3)
                .build();
            
            physics_world.collider_set.insert_with_parent(
                collider,
                handle,
                &mut physics_world.rigid_body_set,
            );
        }

        // Create joints
        for i in 1..rigid_body_handles.len() {
            let joint = RevoluteJointBuilder::new()
                .local_anchor1(point![0.3, 0.0])  // 15 pixels = 0.3 meters
                .local_anchor2(point![-0.3, 0.0])
                .limits([-0.5, 0.5])  // Tighter rotation limits
                .build();
            
            let handle = physics_world.joint_set.insert(
                rigid_body_handles[i - 1],
                rigid_body_handles[i],
                joint,
                true,
            );
            joint_handles.push(handle);
        }

        Self {
            segments,
            target_segments: 10,
            show_properties: false,
            show_skin: true,  // Enable skin by default
            physics_world,
            rigid_body_handles,
            joint_handles,
            time: 0.0,
            startup_delay: 1.0,  // 1 second delay before applying forces
        }
    }
}

impl Creature for TestChain {
    fn update_state(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.unstable_dt);
        if dt > 0.0 {
            self.time += dt;
            self.startup_delay -= dt;

            // Only apply motion after startup delay
            if self.startup_delay <= 0.0 {
                // Get cursor position in screen coordinates
                if let Some(cursor_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                    if let Some(head_handle) = self.rigid_body_handles.first() {
                        if let Some(head) = self.physics_world.rigid_body_set.get_mut(*head_handle) {
                            // Convert cursor position to physics world coordinates
                            let target_pos = vector![
                                cursor_pos.x / PIXELS_PER_METER,
                                cursor_pos.y / PIXELS_PER_METER
                            ];
                            
                            // Get current head position
                            let current_pos = head.translation();
                            
                            // Calculate direction to cursor
                            let direction = (target_pos - current_pos).normalize();
                            
                            // Set velocity towards cursor with some damping
                            let speed = 5.0; // meters per second
                            let velocity = direction * speed;
                            
                            // Apply velocity with some damping
                            head.set_linvel(velocity, true);
                            
                            // Add some angular damping to prevent excessive rotation
                            head.set_angvel(0.0, true);
                        }
                    }
                }
            }

            // Step physics with a fixed timestep for stability
            self.physics_world.step(1.0/60.0);

            // Update segment positions
            for (i, handle) in self.rigid_body_handles.iter().enumerate() {
                if let Some(body) = self.physics_world.rigid_body_set.get(*handle) {
                    let pos = body.translation();
                    // Convert from meters back to pixels
                    self.segments[i].pos = egui::Pos2::new(
                        pos.x * PIXELS_PER_METER,
                        pos.y * PIXELS_PER_METER
                    );
                    
                    // Update side points
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

            // Request continuous repaint for smooth animation
            ctx.request_repaint();
        }
    }

    fn draw(&self, painter: &egui::Painter) {
        // Pre-allocate vectors for better performance
        let mut shapes = Vec::with_capacity(self.segments.len() * 2);
        
        // Draw the skeleton first
        for segment in &self.segments {
            // Add main circle
            shapes.push(egui::Shape::circle_filled(
                segment.pos,
                segment.radius,
                segment.color,
            ));
        }

        // Add connecting lines
        for i in 0..self.segments.len() - 1 {
            shapes.push(egui::Shape::line_segment(
                [self.segments[i].pos, self.segments[i + 1].pos],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 100)),
            ));
        }

        // Draw skin if enabled
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
        "Test Chain"
    }

    fn setup_physics(&mut self) {
        // Not needed for this test
    }

    fn update_physics(&mut self, dt: f32) {
        self.physics_world.step(dt);
    }

    fn get_rigid_body_handles(&self) -> &[RigidBodyHandle] {
        &self.rigid_body_handles
    }

    fn get_joint_handles(&self) -> &[ImpulseJointHandle] {
        &self.joint_handles
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl eframe::App for TestChain {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::drag(),
            );

            if response.rect.width() > 0.0 && response.rect.height() > 0.0 {
                self.update_state(ctx);
            }

            self.draw(&painter);
        });
    }
} 