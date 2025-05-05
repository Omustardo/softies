use rapier2d::prelude::{RigidBodyHandle, ImpulseJointHandle, RigidBodySet, ImpulseJointSet, ColliderSet};
use nalgebra::Vector2; // Added for vector math in helper

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

    /// Applies custom physics forces (e.g., hydrodynamics) to the creature.
    /// Called after behavior updates, before the main physics step.
    /// Default implementation does nothing.
    fn apply_custom_forces(&self, _rigid_body_set: &mut RigidBodySet) {
        // Default: Do nothing. Creatures needing special forces will override this.
    }
}

// NEW TRAIT for applying custom physics forces
pub trait CustomPhysicsApplier {
    /// Applies custom forces (like hydrodynamics) to the creature's rigid bodies.
    /// This is typically called *after* internal behavior updates but *before* the physics step.
    fn apply_custom_forces(
        &self,
        rigid_body_set: &mut RigidBodySet,
        // Consider adding dt if forces are time-dependent in ways not handled by Rapier's integrator
        // dt: f32,
    );

    /// Helper function to apply anisotropic drag.
    /// Can be called by implementations of `apply_custom_forces`.
    /// Drag Force = -coeff * velocity_component * |velocity_component| * direction_vector
    fn apply_anisotropic_drag(
        body_handle: RigidBodyHandle,
        rigid_body_set: &mut RigidBodySet,
        perp_drag_coeff: f32, // Higher resistance perpendicular to body segment
        forward_drag_coeff: f32, // Lower resistance parallel to body segment
    ) {
       if let Some(body) = rigid_body_set.get_mut(body_handle) {
            let linvel = *body.linvel();
            // Ensure velocity is not NaN or infinite, which can cause issues
            if !linvel.x.is_finite() || !linvel.y.is_finite() {
                return;
            }

            let angle = body.rotation().angle();
            let forward_dir = Vector2::new(angle.cos(), angle.sin());
            let right_dir = Vector2::new(-angle.sin(), angle.cos()); // Perpendicular

            let v_forward = linvel.dot(&forward_dir);
            let v_perpendicular = linvel.dot(&right_dir);

            // Quadratic drag model: F = -k * v * |v|
            // Ensure coefficients are non-negative
            let safe_perp_coeff = perp_drag_coeff.max(0.0);
            let safe_forward_coeff = forward_drag_coeff.max(0.0);

            let drag_force_perp_magnitude = safe_perp_coeff * v_perpendicular * v_perpendicular.abs();
            let drag_force_forward_magnitude = safe_forward_coeff * v_forward * v_forward.abs();

            // Calculate force vectors
            let drag_force_perp = -drag_force_perp_magnitude * right_dir;
            let drag_force_forward = -drag_force_forward_magnitude * forward_dir;

            // Apply forces if they are finite
            if drag_force_perp.x.is_finite() && drag_force_perp.y.is_finite() {
                body.add_force(drag_force_perp, true);
            }
            if drag_force_forward.x.is_finite() && drag_force_forward.y.is_finite() {
                body.add_force(drag_force_forward, true);
            }
        }
    }
}
