use rapier2d::prelude::{RigidBodyHandle, ImpulseJointHandle, RigidBodySet, ImpulseJointSet, ColliderSet};
use nalgebra::Vector2; // Added for vector math in helper
use eframe::egui; // Added for Painter in draw method

use crate::creature_attributes::CreatureAttributes;

/// Represents the general behavioral state of a creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatureState {
    Idle,      // Doing nothing specific, minimal movement.
    Wandering, // Exploring randomly.
    Resting,   // Actively recovering energy.
    SeekingFood, // Includes plankton seeking light
    Fleeing,
    // Add more states as needed (e.g., Eating, Mating)
}

/// Context about the simulation world passed to creature updates.
#[derive(Debug, Clone, Copy)]
pub struct WorldContext {
    pub world_height: f32,
    pub pixels_per_meter: f32,
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
        world_context: &WorldContext, // Changed parameter
        // Add other context later, e.g., sensing results: &SensingData
    );

    /// Applies custom physics forces (e.g., hydrodynamics) to the creature.
    /// Called after behavior updates, before the main physics step.
    /// Default implementation does nothing.
    fn apply_custom_forces(&self, _rigid_body_set: &mut RigidBodySet) {
        // Default: Do nothing. Creatures needing special forces will override this.
    }

    /// Draws the creature onto the screen using egui.
    fn draw(
        &self,
        painter: &egui::Painter,
        rigid_body_set: &RigidBodySet,
        world_to_screen: &dyn Fn(Vector2<f32>) -> egui::Pos2,
        zoom: f32,
        is_hovered: bool,
        pixels_per_meter: f32, // Added parameter
    );
}
