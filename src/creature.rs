// Remove Bevy imports
// use bevy::prelude::*;
// use bevy_rapier2d::prelude::*;
use rapier2d::prelude::{RigidBodyHandle, ImpulseJointHandle};

pub trait Creature {
    // Return slices of Rapier handles
    fn get_rigid_body_handles(&self) -> &[RigidBodyHandle];
    fn get_joint_handles(&self) -> &[ImpulseJointHandle];
    // Maybe add methods for drawing later, e.g., get_color(), get_radius()
}

// Remove Bevy-specific Segment component for now.
// The app will query Rapier data directly for drawing.
// #[derive(Component)]
// pub struct Segment {
//     pub radius: f32,
//     pub color: Color,
// }
//
// impl Segment {
//     pub fn new(radius: f32, color: Color) -> Self {
//         Self {
//             radius,
//             color,
//         }
//     }
// } 