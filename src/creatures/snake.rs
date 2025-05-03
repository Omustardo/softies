use rapier2d::prelude::*;
use nalgebra::{Point2, Vector2};

use crate::creature::{Creature, CreatureState}; // Keep crate:: for sibling module
use crate::creature_attributes::{CreatureAttributes, DietType}; // Use package name

pub struct Snake {
    segment_handles: Vec<RigidBodyHandle>,
    joint_handles: Vec<ImpulseJointHandle>,
    pub segment_radius: f32, // Made public for drawing access in app.rs
    segment_count: usize,
    segment_spacing: f32,
    wiggle_timer: f32, // Timer to control the wiggle animation
    attributes: CreatureAttributes, // Added attributes field
    current_state: CreatureState, // Added state field
}

impl Snake {
    // Simple constructor
    pub fn new(segment_radius: f32, segment_count: usize, segment_spacing: f32) -> Self {
        // Calculate a rough size based on segments
        let size = segment_count as f32 * segment_spacing;
        // Placeholder attributes for a snake
        let attributes = CreatureAttributes::new(
            100.0,                // max_energy
            5.0,                  // energy_recovery_rate
            100.0,                // max_satiety
            1.0,                  // metabolic_rate
            DietType::Carnivore,  // diet_type (let's make it a carnivore for now)
            size,                 // size
            vec!["small_fish".to_string(), "worm".to_string()], // prey_tags
            vec!["snake".to_string(), "medium_predator".to_string()], // self_tags
        );

        Self {
            segment_handles: Vec::with_capacity(segment_count),
            joint_handles: Vec::with_capacity(segment_count.saturating_sub(1)),
            segment_radius,
            segment_count,
            segment_spacing,
            wiggle_timer: 0.0, // Initialize timer
            attributes,        // Initialize attributes
            current_state: CreatureState::Wandering, // Start wandering
        }
    }

    // Renamed from spawn, takes Rapier sets as arguments
    pub fn spawn_rapier(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        impulse_joint_set: &mut ImpulseJointSet,
        initial_position: Vector2<f32>,
        creature_id: u128, // Added creature ID
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
                .linear_damping(0.8) // Reduced damping
                .angular_damping(1.0) // Simulate rotational drag
                .build();
            let segment_handle = rigid_body_set.insert(rb);
            self.segment_handles.push(segment_handle);

            // Create Collider
            let collider = ColliderBuilder::ball(self.segment_radius)
                             .restitution(0.2) // Lower restitution for less bounce
                             .density(100.0) // Significantly higher density (closer to water-like mass)
                             .user_data(creature_id) // Set the creature ID here
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

    // Helper function for the wiggle/movement logic
    fn apply_wiggle(
        &mut self,
        dt: f32,
        impulse_joint_set: &mut ImpulseJointSet,
        amplitude_scale: f32, // Scale factor for wiggle intensity
        frequency_scale: f32, // Scale factor for wiggle speed
        energy_cost_scale: f32, // Scale factor for energy cost
    ) {
        self.wiggle_timer += dt * 3.0 * frequency_scale;

        let target_velocity_amplitude = 1.8 * amplitude_scale; // Slightly reduced base amplitude
        let wiggle_frequency = 1.5;
        let motor_force_factor = 4.0; // Reduced from 10.0
        let base_energy_cost_per_rad_per_sec = 0.5;

        let mut total_applied_velocity = 0.0;

        for (i, handle) in self.joint_handles.iter().enumerate() {
            if let Some(joint) = impulse_joint_set.get_mut(*handle) {
                let phase = self.wiggle_timer + (i as f32 / (self.segment_count - 1) as f32) * std::f32::consts::TAU * wiggle_frequency;
                let target_velocity = phase.sin() * target_velocity_amplitude;
                joint.data.set_motor_velocity(JointAxis::AngX, target_velocity, motor_force_factor);
                total_applied_velocity += target_velocity.abs();
            }
        }

        let energy_consumed = total_applied_velocity * base_energy_cost_per_rad_per_sec * energy_cost_scale * dt;
        self.attributes.consume_energy(energy_consumed);
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

    // Implement required methods
    fn attributes(&self) -> &CreatureAttributes {
        &self.attributes
    }

    fn attributes_mut(&mut self) -> &mut CreatureAttributes {
        &mut self.attributes
    }

    fn drawing_radius(&self) -> f32 {
        self.segment_radius
    }

    fn type_name(&self) -> &'static str {
        "Snake"
    }

    fn current_state(&self) -> CreatureState {
        self.current_state
    }

    fn update_state_and_behavior(
        &mut self,
        dt: f32,
        _rigid_body_set: &mut RigidBodySet, // Prefix with underscore to silence warning
        impulse_joint_set: &mut ImpulseJointSet,
        _collider_set: &ColliderSet, // Added, prefixed with underscore for now
    ) {
        // --- State Transition Logic --- 
        let mut next_state = self.current_state; // Start with current state

        // Priorities: Fleeing > SeekingFood > Resting > Wandering > Idle 
        // (We only have Resting and Wandering/Idle logic for now)

        if self.attributes.is_tired() {
            next_state = CreatureState::Resting;
        } else if self.attributes.is_hungry() {
             // TODO: Add sensing check here. If food nearby, switch to SeekingFood
             // For now, just keep wandering even if hungry, until we have sensing.
             if self.current_state == CreatureState::Resting { 
                 // If rested enough, start wandering again
                 if self.attributes.energy > self.attributes.max_energy * 0.5 { // Example threshold to stop resting
                     next_state = CreatureState::Wandering;
                 }
             } else { // If not resting, default to wandering
                 next_state = CreatureState::Wandering;
             }
        } else { // Not tired, not hungry
             if self.current_state == CreatureState::Resting { 
                 // If rested enough, start wandering again
                 if self.attributes.energy > self.attributes.max_energy * 0.8 { // Higher threshold to stop resting if not hungry
                     next_state = CreatureState::Wandering;
                 }
             } else { // If not resting, default to wandering
                 next_state = CreatureState::Wandering;
             }
        }
        // TODO: Add transition logic for Fleeing based on sensed predators
        
        self.current_state = next_state;

        // --- Execute Behavior based on State --- 
        match self.current_state {
            CreatureState::Idle => {
                // Minimal movement or stop motors completely
                self.apply_wiggle(dt, impulse_joint_set, 0.1, 0.5, 0.1); // Very slow, low cost
            }
            CreatureState::Wandering => {
                // Standard wiggle - Increased frequency scale
                self.apply_wiggle(dt, impulse_joint_set, 1.5, 1.5, 1.5); // Increased frequency_scale (1.0->1.5)
            }
            CreatureState::Resting => {
                // No active movement, energy recovery happens passively in App::update
                 // Ensure motors are stopped if they were active
                 for handle in self.joint_handles.iter() {
                     if let Some(joint) = impulse_joint_set.get_mut(*handle) {
                         joint.data.set_motor_velocity(JointAxis::AngX, 0.0, 0.0);
                     }
                 }
            }
            CreatureState::SeekingFood => {
                // TODO: Implement movement towards food
                // For now, just wander
                self.apply_wiggle(dt, impulse_joint_set, 1.0, 1.2, 1.0); // Slightly faster wiggle, standard cost
            }
            CreatureState::Fleeing => {
                // TODO: Implement movement away from predator
                // For now, just wander fast
                self.apply_wiggle(dt, impulse_joint_set, 1.5, 1.5, 1.5); // Fast, high cost wiggle
            }
        }
    }
} 