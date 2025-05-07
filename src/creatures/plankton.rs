use rapier2d::prelude::*;
use nalgebra::{Vector2, Point2};
use eframe::egui; // Keep for draw method later
use rand::Rng;

use crate::creature::{Creature, CreatureState, WorldContext};
use crate::creature_attributes::{CreatureAttributes, DietType};

pub struct Plankton {
    segment_handles: Vec<RigidBodyHandle>, // Changed from single handle
    joint_handle: Option<ImpulseJointHandle>, // Added joint handle
    attributes: CreatureAttributes,
    current_state: CreatureState,
    pub primary_radius: f32, // Renamed from radius
    pub secondary_radius: f32, // Added second radius
}

impl Plankton {
    // Constructor
    pub fn new(primary_radius: f32) -> Self {
        let secondary_radius = primary_radius * 0.6; // Smaller second segment
        let size = primary_radius * 2.0; // Base size on primary segment

        let attributes = CreatureAttributes::new(
            20.0,                // max_energy (low)
            1.0,                 // energy_recovery_rate
            50.0,                // max_satiety
            0.1,                 // metabolic_rate
            DietType::Herbivore, // Placeholder
            size,
            vec![],
            vec!["plankton".to_string(), "small_food".to_string()],
        );

        Self {
            segment_handles: Vec::with_capacity(2),
            joint_handle: None,
            attributes,
            current_state: CreatureState::Wandering,
            primary_radius,
            secondary_radius,
        }
    }

    // Spawn method
    pub fn spawn_rapier(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        impulse_joint_set: &mut ImpulseJointSet, // Added joint set param
        initial_position: Vector2<f32>,
        creature_id: u128,
    ) {
        self.segment_handles.clear();
        self.joint_handle = None;

        let segment_distance = (self.primary_radius + self.secondary_radius) * 0.8; // How far apart segments start

        // --- Create Primary Segment --- 
        let rb1 = RigidBodyBuilder::dynamic()
            .translation(initial_position)
            .linear_damping(5.0)
            .angular_damping(3.0) // Slightly more angular damping
            .gravity_scale(0.8)
            .ccd_enabled(true) // Enabled CCD
            .build();
        let handle1 = rigid_body_set.insert(rb1);
        self.segment_handles.push(handle1);

        let collider1 = ColliderBuilder::ball(self.primary_radius)
                         .restitution(0.1)
                         .density(10.0)
                         .user_data(creature_id)
                         .build();
        collider_set.insert_with_parent(collider1, handle1, rigid_body_set);

        // --- Create Secondary Segment --- 
        let pos2 = initial_position + Vector2::y() * segment_distance;
        let rb2 = RigidBodyBuilder::dynamic()
            .translation(pos2)
            .linear_damping(5.0)
            .angular_damping(3.0)
            .gravity_scale(0.8)
            .ccd_enabled(true) // Enabled CCD
            .build();
        let handle2 = rigid_body_set.insert(rb2);
        self.segment_handles.push(handle2);

        let collider2 = ColliderBuilder::ball(self.secondary_radius)
                         .restitution(0.1)
                         .density(10.0)
                         .user_data(creature_id)
                         .build();
        collider_set.insert_with_parent(collider2, handle2, rigid_body_set);

        // --- Create Joint --- 
        // Connect the two segments
        let joint = RevoluteJointBuilder::new()
            .local_anchor1(Point2::new(0.0, segment_distance / 2.0)) // Adjusted anchors for segment distance
            .local_anchor2(Point2::new(0.0, -segment_distance / 2.0))
            .motor_model(MotorModel::ForceBased) // Use force-based model
            .motor_velocity(0.0, 0.0) // Target zero relative velocity
            .motor_max_force(5.0) // Low force to allow some flex but keep them together
            .limits([-0.1, 0.1]) // Very small rotation limit if needed
            .build();
        self.joint_handle = Some(impulse_joint_set.insert(handle1, handle2, joint, true));
    }

    // Apply buoyancy and drag
    fn apply_buoyancy_and_drag(
        &self,
        rigid_body_set: &mut RigidBodySet,
    ) {
        let buoyancy_force_magnitude_seeking = 0.5; // Slow rise
        let downward_force_magnitude_resting = 0.1; // Gentle sink

        for handle in &self.segment_handles {
            if let Some(body) = rigid_body_set.get_mut(*handle) {
                match self.current_state {
                    CreatureState::SeekingFood => {
                        body.add_force(Vector2::new(0.0, buoyancy_force_magnitude_seeking), true);
                    }
                    CreatureState::Wandering => {
                        // Small buoyant force to roughly maintain depth, or slight rise
                        body.add_force(Vector2::new(0.0, buoyancy_force_magnitude_seeking * 0.5), true);
                    }
                    CreatureState::Resting | CreatureState::Idle => {
                        // Apply a gentle downward force to sink
                        body.add_force(Vector2::new(0.0, -downward_force_magnitude_resting), true);
                    }
                    CreatureState::Fleeing => {
                        // Potentially faster movement, or rely on impulses
                    }
                }
                // Optional drag could be applied per-segment here too
            }
        }
    }
}

impl Creature for Plankton {
    fn get_rigid_body_handles(&self) -> &[RigidBodyHandle] {
        &self.segment_handles // Return the vec slice
    }

    fn get_joint_handles(&self) -> &[ImpulseJointHandle] {
        // Convert the Option<Handle> to a slice of 0 or 1 elements
        self.joint_handle.as_slice()
    }

    fn attributes(&self) -> &CreatureAttributes {
        &self.attributes
    }

    fn attributes_mut(&mut self) -> &mut CreatureAttributes {
        &mut self.attributes
    }

    fn drawing_radius(&self) -> f32 {
        self.primary_radius // Return the main radius for simple highlighting etc.
    }

    fn type_name(&self) -> &'static str {
        "Plankton"
    }

    fn current_state(&self) -> CreatureState {
        self.current_state
    }

    fn update_state_and_behavior(
        &mut self,
        _dt: f32,
        rigid_body_set: &mut RigidBodySet,
        _impulse_joint_set: &mut ImpulseJointSet,
        _collider_set: &ColliderSet,
        world_context: &WorldContext,
    ) {
        // State transition logic - use primary segment for position check
        let primary_handle = self.segment_handles.get(0).cloned().unwrap_or_else(RigidBodyHandle::invalid);
        let current_y = rigid_body_set
            .get(primary_handle)
            .map_or(0.0, |b| b.translation().y);

        let top_zone_threshold = world_context.world_height * 0.3;
        let energy_threshold_wandering = self.attributes.max_energy * 0.8;
        let next_state = if self.attributes.is_tired() {
            CreatureState::Resting
        } else {
            if current_y < top_zone_threshold {
                CreatureState::SeekingFood
            } else if current_y >= top_zone_threshold && self.attributes.energy >= energy_threshold_wandering {
                CreatureState::Wandering
            } else {
                CreatureState::SeekingFood
            }
        };
        self.current_state = next_state;

        // --- Execute Behavior based on State --- 
        match self.current_state {
            CreatureState::Wandering => {
                // Apply small random impulse to the primary segment
                 if let Some(body) = rigid_body_set.get_mut(primary_handle) {
                    let mut rng = rand::thread_rng();
                    let impulse_strength = 0.05;
                    let impulse = Vector2::new(
                        rng.gen_range(-impulse_strength..impulse_strength),
                        rng.gen_range(-impulse_strength..impulse_strength)
                    );
                    body.apply_impulse(impulse, true);
                 }
            }
            CreatureState::SeekingFood => { /* Buoyancy handles upward movement */ }
            CreatureState::Resting => { /* Buoyancy handles sinking */ }
            CreatureState::Idle => { /* Do nothing */}
            CreatureState::Fleeing => { /* Do nothing */}
        }
    }

    fn apply_custom_forces(&self, rigid_body_set: &mut RigidBodySet) {
        // Call the helper method
        self.apply_buoyancy_and_drag(rigid_body_set); // Pass the set
    }

    fn draw(
        &self,
        painter: &egui::Painter,
        rigid_body_set: &RigidBodySet,
        world_to_screen: &dyn Fn(Vector2<f32>) -> egui::Pos2,
        zoom: f32,
        is_hovered: bool,
        pixels_per_meter: f32,
    ) {
        let base_color = match self.current_state() {
            CreatureState::Idle => egui::Color32::from_rgb(100, 120, 100), // Dull Greenish
            CreatureState::Wandering => egui::Color32::from_rgb(120, 180, 120), // Soft Green
            CreatureState::Resting => egui::Color32::from_rgb(80, 100, 80),   // Darker, Duller Green
            CreatureState::SeekingFood => egui::Color32::from_rgb(150, 220, 150), // Brighter Green
            CreatureState::Fleeing => egui::Color32::TRANSPARENT, // Keep transparent or choose panic color
        };

        let handles = self.get_rigid_body_handles();
        if handles.len() != 2 { 
            // Fallback: Draw simple circles if we don't have exactly 2 segments
            let screen_radius = self.primary_radius * pixels_per_meter * zoom;
            for handle in handles {
                if let Some(body) = rigid_body_set.get(*handle) {
                    let screen_pos = world_to_screen(*body.translation());
                    painter.circle_filled(screen_pos, screen_radius, base_color);
                }
            }
            return; 
        }

        // Get positions
        let pos1 = rigid_body_set.get(handles[0]).map(|b| *b.translation());
        let pos2 = rigid_body_set.get(handles[1]).map(|b| *b.translation());

        if let (Some(p1), Some(p2)) = (pos1, pos2) {
            let radius1 = self.primary_radius;
            let radius2 = self.secondary_radius;

            // Calculate direction and perpendicular vectors
            let direction = (p2 - p1).try_normalize(1e-6).unwrap_or_else(Vector2::zeros);
            let perpendicular = Vector2::new(-direction.y, direction.x);

            // Calculate the 4 points for the quadrilateral skin in world coordinates
            let skin_world = [
                p1 + perpendicular * radius1, // Top-leftish
                p2 + perpendicular * radius2, // Top-rightish
                p2 - perpendicular * radius2, // Bottom-rightish
                p1 - perpendicular * radius1, // Bottom-leftish
            ];

            // Convert to screen coordinates
            let skin_screen: Vec<egui::Pos2> = skin_world
                .into_iter()
                .map(|wp| world_to_screen(wp))
                .collect();

            if skin_screen.len() == 4 {
                // Draw highlight outline
                if is_hovered {
                    // Use average screen radius for highlight stroke thickness
                    let avg_screen_radius = (radius1 + radius2) / 2.0 * pixels_per_meter * zoom;
                    painter.add(egui::Shape::convex_polygon(
                        skin_screen.clone(),
                        egui::Color32::TRANSPARENT,
                        egui::Stroke::new(avg_screen_radius * 0.4, egui::Color32::WHITE),
                    ));
                }
                // Draw the main skin polygon
                painter.add(egui::Shape::convex_polygon(
                    skin_screen,
                    base_color,
                    egui::Stroke::NONE,
                ));
            }
        } else {
            // Fallback if bodies not found (draw circles)
            let screen_radius1 = self.primary_radius * pixels_per_meter * zoom;
            let screen_radius2 = self.secondary_radius * pixels_per_meter * zoom;
             if let Some(body) = rigid_body_set.get(handles[0]) {
                 let screen_pos = world_to_screen(*body.translation());
                 painter.circle_filled(screen_pos, screen_radius1, base_color);
             }
              if let Some(body) = rigid_body_set.get(handles[1]) {
                 let screen_pos = world_to_screen(*body.translation());
                 painter.circle_filled(screen_pos, screen_radius2, base_color);
             }
        }
    }
} 