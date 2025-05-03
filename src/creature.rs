use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub trait Creature {
    fn get_rigid_body_handles(&self) -> &[Entity];
    fn get_joint_handles(&self) -> &[Entity];
}

#[derive(Component)]
pub struct Segment {
    pub radius: f32,
    pub color: Color,
}

impl Segment {
    pub fn new(radius: f32, color: Color) -> Self {
        Self {
            radius,
            color,
        }
    }
} 