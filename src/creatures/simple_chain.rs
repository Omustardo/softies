use eframe::egui;
use rapier2d::prelude::*;
use crate::creature::{Creature, Segment, PhysicsWorld};
use std::any::Any;

const PIXELS_PER_METER: f32 = 50.0;

pub struct SimpleChain {
    segments: Vec<Segment>,
    physics_world: PhysicsWorld,
    rigid_body_handles: Vec<RigidBodyHandle>,
    joint_handles: Vec<ImpulseJointHandle>,
    time: f32,
    startup_delay: f32,  // Add startup delay
}

impl Default for SimpleChain {
    fn default() -> Self {
        let mut segments = Vec::new();
        let start_pos = egui::Pos2::new(400.0, 300.0);
        let mut current_pos = start_pos;

        // Create 5 segments
        for i in 0..5 {
            segments.push(Segment::new(
                current_pos,
                10.0,  // All segments same size
                if i == 0 {
                    egui::Color32::from_rgb(200, 100, 100)  // Red for head
                } else {
                    egui::Color32::from_rgb(100, 200, 100)  // Green for body
                },
            ));
            current_pos = current_pos + egui::Vec2::new(20.0, 0.0);  // 20 pixels spacing
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
                .local_anchor1(point![0.2, 0.0])  // 10 pixels = 0.2 meters
                .local_anchor2(point![-0.2, 0.0])
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
            physics_world,
            rigid_body_handles,
            joint_handles,
            time: 0.0,
            startup_delay: 1.0,  // 1 second delay before applying forces
        }
    }
}

impl Creature for SimpleChain {
    fn update_state(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.unstable_dt);
        if dt > 0.0 {
            self.time += dt;
            self.startup_delay -= dt;

            // Only apply motion after startup delay
            if self.startup_delay <= 0.0 {
                // Apply a simple circular motion to the head using velocity control
                if let Some(head_handle) = self.rigid_body_handles.first() {
                    if let Some(head) = self.physics_world.rigid_body_set.get_mut(*head_handle) {
                        let angle = self.time * 1.0;  // Slower rotation
                        // Set velocity directly instead of applying forces
                        let velocity = vector![
                            angle.cos() * 2.0,  // 2 meters per second
                            angle.sin() * 1.0   // 1 meter per second
                        ];
                        head.set_linvel(velocity, true);
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
        // Draw segments
        for segment in &self.segments {
            painter.circle_filled(segment.pos, segment.radius, segment.color);
        }

        // Draw connecting lines
        for i in 0..self.segments.len() - 1 {
            painter.line_segment(
                [self.segments[i].pos, self.segments[i + 1].pos],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 100)),
            );
        }
    }

    fn get_segments(&self) -> &[Segment] {
        &self.segments
    }

    fn get_segments_mut(&mut self) -> &mut [Segment] {
        &mut self.segments
    }

    fn get_target_segments(&self) -> usize {
        self.segments.len()
    }

    fn set_target_segments(&mut self, _count: usize) {
        // Not implemented for simple chain
    }

    fn get_show_properties(&self) -> bool {
        false
    }

    fn set_show_properties(&mut self, _show: bool) {
        // Not implemented for simple chain
    }

    fn get_show_skin(&self) -> bool {
        false
    }

    fn set_show_skin(&mut self, _show: bool) {
        // Not implemented for simple chain
    }

    fn get_type_name(&self) -> &'static str {
        "Simple Chain"
    }

    fn setup_physics(&mut self) {
        // Not needed for simple chain
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