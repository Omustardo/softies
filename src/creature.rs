use rapier2d::prelude::{RigidBodyHandle, ImpulseJointHandle, RigidBodySet, ImpulseJointSet, ColliderSet, QueryPipeline};
use nalgebra::Vector2; // Added for vector math in helper
use eframe::egui; // Added for Painter in draw method

use crate::creature_attributes::CreatureAttributes;

/// Represents the general behavioral state of a creature.
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct WorldContext {
    pub world_height: f32,
    pub pixels_per_meter: f32,
}

/// Basic information about a creature, used for awareness by other creatures.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CreatureInfo {
    pub id: u128,
    pub creature_type_name: &'static str,
    pub primary_body_handle: RigidBodyHandle, // Or Option<RigidBodyHandle> if a creature might not have one temporarily
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub radius: f32, // General radius for interaction/sensing
    // pub attributes: CreatureAttributes, // Consider if the full attributes are needed or just specific parts like size/tags
}

#[allow(dead_code)]
pub trait Creature {
    // Return unique ID for this creature instance
    fn id(&self) -> u128;

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
        own_id: u128, // ID of the creature instance being updated
        rigid_body_set: &mut RigidBodySet, // Still mutable for direct actions by self
        impulse_joint_set: &mut ImpulseJointSet, // Still mutable for direct actions by self
        collider_set: &ColliderSet, // Immutable for querying others
        query_pipeline: &QueryPipeline, // For spatial queries
        all_creatures_info: &Vec<CreatureInfo>, // Info about all other creatures
        world_context: &WorldContext,
    );

    /// Applies custom physics forces (e.g., hydrodynamics) to the creature.
    /// Called after behavior updates, before the main physics step.
    /// Default implementation does nothing.
    fn apply_custom_forces(&self, _rigid_body_set: &mut RigidBodySet, _world_context: &WorldContext) {
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
