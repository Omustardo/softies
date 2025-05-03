use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use crate::creatures::snake::Snake;

mod creature;
mod creatures;

// Constants for the aquarium
const AQUARIUM_WIDTH: f32 = 1000.0;
const AQUARIUM_HEIGHT: f32 = 800.0;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 5.0;
const CAMERA_BOUND_PADDING: f32 = 0.3; // 30% padding

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(PanCamPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Calculate camera bounds
    let max_x = AQUARIUM_WIDTH / 2.0 * (1.0 + CAMERA_BOUND_PADDING);
    let min_x = -max_x;
    let max_y = AQUARIUM_HEIGHT / 2.0 * (1.0 + CAMERA_BOUND_PADDING);
    let min_y = -max_y;

    // Camera with PanCam component
    commands.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Left],
            enabled: true,
            zoom_to_cursor: true,
            min_scale: MIN_ZOOM,
            max_scale: Some(MAX_ZOOM),
            min_x: Some(min_x),
            max_x: Some(max_x),
            min_y: Some(min_y),
            max_y: Some(max_y),
            ..default()
        }
    ));

    // Create aquarium boundaries
    let wall_thickness = 20.0;
    
    // Left wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(wall_thickness / 2.0, AQUARIUM_HEIGHT / 2.0),
        TransformBundle::from(Transform::from_xyz(-AQUARIUM_WIDTH / 2.0, 0.0, 0.0)),
    ));

    // Right wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(wall_thickness / 2.0, AQUARIUM_HEIGHT / 2.0),
        TransformBundle::from(Transform::from_xyz(AQUARIUM_WIDTH / 2.0, 0.0, 0.0)),
    ));

    // Top wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(AQUARIUM_WIDTH / 2.0, wall_thickness / 2.0),
        TransformBundle::from(Transform::from_xyz(0.0, AQUARIUM_HEIGHT / 2.0, 0.0)),
    ));

    // Bottom wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(AQUARIUM_WIDTH / 2.0, wall_thickness / 2.0),
        TransformBundle::from(Transform::from_xyz(0.0, -AQUARIUM_HEIGHT / 2.0, 0.0)),
    ));

    // Spawn snake
    let mut snake = Snake::new(10.0, 10, 20.0);
    snake.spawn(&mut commands);
} 