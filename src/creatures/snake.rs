// Remove Bevy imports
// use bevy::prelude::*;
// use bevy_rapier2d::prelude::*;
use rapier2d::prelude::*;
use nalgebra::{Point2, Vector2};

use crate::creature::Creature;

// Remove Bevy component derive
// #[derive(Component)]
pub struct Snake {
    // Store Rapier handles instead of Bevy Entities
    segment_handles: Vec<RigidBodyHandle>,
    joint_handles: Vec<ImpulseJointHandle>,
    pub segment_radius: f32, // Made public for drawing access in app.rs
    segment_count: usize,
    segment_spacing: f32,
    wiggle_timer: f32, // Timer to control the wiggle animation
}

// Remove Default impl as it requires physics context
// impl Default for Snake { ... }

impl Snake {
    // Simple constructor
    pub fn new(segment_radius: f32, segment_count: usize, segment_spacing: f32) -> Self {
        Self {
            segment_handles: Vec::with_capacity(segment_count),
            joint_handles: Vec::with_capacity(segment_count.saturating_sub(1)),
            segment_radius,
            segment_count,
            segment_spacing,
            wiggle_timer: 0.0, // Initialize timer
        }
    }

    // Renamed from spawn, takes Rapier sets as arguments
    pub fn spawn_rapier(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        impulse_joint_set: &mut ImpulseJointSet,
        initial_position: Vector2<f32>, // Added parameter for initial position
    ) {
        self.segment_handles.clear();
        self.joint_handles.clear();

        let mut parent_handle: Option<RigidBodyHandle> = None;

        for i in 0..self.segment_count {
            let segment_x = initial_position.x + i as f32 * self.segment_spacing;
            let segment_y = initial_position.y;

            // Create RigidBody
            let rb = RigidBodyBuilder::dynamic()
                .translation(vector![segment_x, segment_y])
                .linear_damping(2.0) // Simulate drag/water resistance
                .angular_damping(1.0) // Simulate rotational drag
                .build();
            let segment_handle = rigid_body_set.insert(rb);
            self.segment_handles.push(segment_handle);

            // Create Collider
            let collider = ColliderBuilder::ball(self.segment_radius)
                             .restitution(0.2) // Lower restitution for less bounce
                             .density(100.0) // Significantly higher density (closer to water-like mass)
                             .build();
            collider_set.insert_with_parent(collider, segment_handle, rigid_body_set);

            // Create joint with the previous segment
            if let Some(prev_handle) = parent_handle {
                let joint = RevoluteJointBuilder::new()
                    // Convert vectors to points for anchors
                    .local_anchor1(Point2::new(self.segment_spacing / 2.0, 0.0))
                    .local_anchor2(Point2::new(-self.segment_spacing / 2.0, 0.0))
                    // Add damping and stiffness for a more "soft" feel
                    .motor_velocity(0.0, 0.0) // Target velocity = 0
                    .motor_max_force(100.0) // Limit motor force
                    .motor_model(MotorModel::ForceBased)
                    //.set_contacts_enabled(false) // Maybe disable segment-segment collision?
                    .build();
                // Insert joint into the provided set
                let joint_handle = impulse_joint_set.insert(prev_handle, segment_handle, joint, true);
                self.joint_handles.push(joint_handle);
            }

            parent_handle = Some(segment_handle);
        }
    }

    // Method to apply wiggle forces using joint motors
    pub fn actuate(&mut self, dt: f32, impulse_joint_set: &mut ImpulseJointSet) {
        self.wiggle_timer += dt * 3.0; // Adjust speed of wiggle here

        // --- MOTOR TUNING PARAMETERS --- >>>> Adjust these! <<<<
        let target_velocity_amplitude = 2.0; // Radians per second
        let wiggle_frequency = 1.5; // How many waves along the body
        // Max force is set during joint creation, but we might need a factor here if set_motor_velocity requires it.
        // Let's assume the factor scales the max_force (1.0 = use full max_force). Check Rapier docs if needed.
        let motor_force_factor = 1.0; 
        // --- END TUNING PARAMETERS ---

        for (i, handle) in self.joint_handles.iter().enumerate() {
            if let Some(joint) = impulse_joint_set.get_mut(*handle) {
                // Calculate target angular velocity based on sine wave
                // Apply phase offset based on position along the snake
                let phase = self.wiggle_timer + (i as f32 / (self.segment_count - 1) as f32) * std::f32::consts::TAU * wiggle_frequency;
                let target_velocity = phase.sin() * target_velocity_amplitude;

                // Set the motor's target velocity.
                // The `set_motor_velocity` function might vary slightly depending on Rapier version.
                // Assuming signature: `set_motor_velocity(target_vel: Real, factor: Real)`
                // where factor might scale the stiffness or max_force.
                // If the joint type is known (e.g., RevoluteJoint), specific methods might exist.
                // We need to ensure the joint *is* a RevoluteJoint implicitly here.
                joint.data.set_motor_velocity(JointAxis::AngX, target_velocity, motor_force_factor);

                // Alternative (if API requires explicit stiffness/damping):
                // joint.data.set_motor(JointAxis::AngX, target_velocity, 0.0, stiffness, damping); // Need axis here too
            }
        }
    }
}

// Remove Bevy component struct
// #[derive(Component)]
// struct SnakeSegment;

impl Creature for Snake {
    fn get_rigid_body_handles(&self) -> &[RigidBodyHandle] {
        &self.segment_handles
    }

    fn get_joint_handles(&self) -> &[ImpulseJointHandle] {
        &self.joint_handles
    }
} 