use eframe::egui;
use rapier2d::prelude::*;
use crate::creature::{Creature, Segment, PhysicsWorld};
use std::any::Any;

pub struct Plankton {
    segment: Segment,
    physics_world: PhysicsWorld,
    rigid_body_handle: RigidBodyHandle,
    collider_handle: ColliderHandle,
    is_eaten: bool,
}

impl Plankton {
    pub fn new(pos: egui::Pos2) -> Self {
        let segment = Segment::new(
            pos,
            5.0,  // Small radius
            egui::Color32::from_rgb(100, 100, 255),  // Blue color
        );

        let mut physics_world = PhysicsWorld::default();
        
        // Create a dynamic rigid body for the plankton
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![pos.x, pos.y])
            .linear_damping(0.5)
            .angular_damping(0.5)
            .build();
        
        let rigid_body_handle = physics_world.rigid_body_set.insert(rigid_body);

        // Create a collider for the plankton
        let collider = ColliderBuilder::ball(segment.radius)
            .restitution(0.2)
            .friction(0.7)
            .sensor(true)  // Make it a sensor so it doesn't physically collide
            .active_events(ActiveEvents::COLLISION_EVENTS)  // Enable collision events
            .build();
        
        let collider_handle = physics_world.collider_set.insert_with_parent(
            collider,
            rigid_body_handle,
            &mut physics_world.rigid_body_set,
        );

        Self {
            segment,
            physics_world,
            rigid_body_handle,
            collider_handle,
            is_eaten: false,
        }
    }

    pub fn respawn(&mut self, pos: egui::Pos2) {
        self.segment.pos = pos;
        self.is_eaten = false;

        // Reset physics body position
        if let Some(rb) = self.physics_world.rigid_body_set.get_mut(self.rigid_body_handle) {
            rb.set_translation(vector![pos.x, pos.y], true);
            rb.set_linvel(vector![0.0, 0.0], true);
            rb.set_angvel(0.0, true);
        }
    }

    pub fn is_eaten(&self) -> bool {
        self.is_eaten
    }

    pub fn mark_as_eaten(&mut self) {
        self.is_eaten = true;
    }

    pub fn get_position(&self) -> egui::Pos2 {
        self.segment.pos
    }

    pub fn get_collider_handle(&self) -> ColliderHandle {
        self.collider_handle
    }
}

impl Creature for Plankton {
    fn update_state(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.unstable_dt);
        if dt > 0.0 && !self.is_eaten {
            // Step physics simulation
            self.physics_world.step(dt);

            // Update position from physics
            if let Some(rb) = self.physics_world.rigid_body_set.get(self.rigid_body_handle) {
                let pos = rb.translation();
                self.segment.pos = egui::Pos2::new(pos.x, pos.y);
            }
        }
    }

    fn draw(&self, painter: &egui::Painter) {
        if !self.is_eaten {
            painter.circle_filled(
                self.segment.pos,
                self.segment.radius,
                self.segment.color,
            );
        }
    }

    fn get_segments(&self) -> &[Segment] {
        std::slice::from_ref(&self.segment)
    }

    fn get_segments_mut(&mut self) -> &mut [Segment] {
        std::slice::from_mut(&mut self.segment)
    }

    fn get_target_segments(&self) -> usize {
        1
    }

    fn set_target_segments(&mut self, _count: usize) {
        // Plankton always has exactly one segment
    }

    fn get_show_properties(&self) -> bool {
        false
    }

    fn set_show_properties(&mut self, _show: bool) {
        // Plankton doesn't need properties panel
    }

    fn get_show_skin(&self) -> bool {
        false
    }

    fn set_show_skin(&mut self, _show: bool) {
        // Plankton doesn't need skin
    }

    fn get_type_name(&self) -> &'static str {
        "Plankton"
    }

    fn setup_physics(&mut self) {
        // Physics is already set up in new()
    }

    fn update_physics(&mut self, dt: f32) {
        if !self.is_eaten {
            self.physics_world.step(dt);
        }
    }

    fn get_rigid_body_handles(&self) -> &[RigidBodyHandle] {
        std::slice::from_ref(&self.rigid_body_handle)
    }

    fn get_joint_handles(&self) -> &[ImpulseJointHandle] {
        &[]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
} 