use rapier2d::prelude::{RigidBodyHandle, ImpulseJointHandle};

pub trait Creature {
    // Return slices of Rapier handles
    fn get_rigid_body_handles(&self) -> &[RigidBodyHandle];
    fn get_joint_handles(&self) -> &[ImpulseJointHandle];
}
