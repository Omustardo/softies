use rapier2d::prelude::{RigidBodyHandle, ImpulseJointHandle, RigidBodySet, ImpulseJointSet, ColliderSet};

use crate::creature_attributes::CreatureAttributes;

/// Represents the general behavioral state of a creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatureState {
    Idle,      // Doing nothing specific, minimal movement.
    Wandering, // Exploring randomly.
    Resting,   // Actively recovering energy.
    SeekingFood,
    Fleeing,
    // Add more states as needed (e.g., Eating, Mating)
}

pub trait Creature {
    // Return slices of Rapier handles
    fn get_rigid_body_handles(&self) -> &[RigidBodyHandle];
    fn get_joint_handles(&self) -> &[ImpulseJointHandle];

    // Access creature attributes
    fn attributes(&self) -> &CreatureAttributes;
    fn attributes_mut(&mut self) -> &mut CreatureAttributes;

    // Drawing info
    fn drawing_radius(&self) -> f32; // Added for drawing

    // Type Info
    fn type_name(&self) -> &'static str; // Added for UI

    // State and Behavior
    fn current_state(&self) -> CreatureState;
    // Decides the next state and executes behavior for the current frame.
    // Needs physics access for actions and potentially sensing.
    fn update_state_and_behavior(
        &mut self,
        dt: f32,
        rigid_body_set: &mut RigidBodySet,
        impulse_joint_set: &mut ImpulseJointSet,
        collider_set: &ColliderSet,
        // Add other context later, e.g., sensing results: &SensingData
    );
}
