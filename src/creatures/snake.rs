use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::creature::Creature;

#[derive(Component)]
pub struct Snake {
    segments: Vec<Entity>,
    segment_radius: f32,
    segment_count: usize,
    segment_spacing: f32,
}

impl Default for Snake {
    fn default() -> Self {
        Self {
            segments: Vec::new(),
            segment_radius: 10.0,
            segment_count: 10,
            segment_spacing: 20.0,
        }
    }
}

impl Snake {
    pub fn new(segment_radius: f32, segment_count: usize, segment_spacing: f32) -> Self {
        Self {
            segments: Vec::new(),
            segment_radius,
            segment_count,
            segment_spacing,
        }
    }

    pub fn spawn(&mut self, commands: &mut Commands) {
        // Clear any existing segments
        self.segments.clear();

        // Spawn segments
        for i in 0..self.segment_count {
            let x = i as f32 * self.segment_spacing;
            let y = 0.0;

            // Spawn segment
            let segment = commands.spawn((
                RigidBody::Dynamic,
                Collider::ball(self.segment_radius),
                TransformBundle::from(Transform::from_xyz(x, y, 0.0)),
                SnakeSegment,
            )).id();

            self.segments.push(segment);

            // Create joint with previous segment if not the first segment
            if i > 0 {
                let parent = self.segments[i - 1];
                
                let joint = RevoluteJointBuilder::new()
                    .local_anchor1(Vec2::new(self.segment_spacing / 2.0, 0.0).into())
                    .local_anchor2(Vec2::new(-self.segment_spacing / 2.0, 0.0).into())
                    .build();

                commands.spawn((
                    ImpulseJoint::new(parent, joint),
                ));
            }
        }
    }
}

#[derive(Component)]
struct SnakeSegment;

impl Creature for Snake {
    fn get_rigid_body_handles(&self) -> &[Entity] {
        &self.segments
    }

    fn get_joint_handles(&self) -> &[Entity] {
        &[]
    }
} 