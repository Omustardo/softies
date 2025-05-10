use rapier2d::prelude::*;
use nalgebra::{Point2, Vector2};
use eframe::egui; // Add egui import

use crate::creature::{Creature, CreatureState, WorldContext, CreatureInfo}; // Add WorldContext and CreatureInfo import
use crate::creature_attributes::{CreatureAttributes, DietType}; // Use package name

pub struct Snake {
    id: u128, // Added creature ID field
    segment_handles: Vec<RigidBodyHandle>,
    joint_handles: Vec<ImpulseJointHandle>,
    pub segment_radius: f32, // Made public for drawing access in app.rs
    segment_count: usize,
    segment_spacing: f32,
    wiggle_timer: f32, // Timer to control the wiggle animation
    attributes: CreatureAttributes, // Added attributes field
    current_state: CreatureState, // Added state field
}

#[allow(dead_code)]
impl Snake {
    // Simple constructor
    pub fn new(segment_radius: f32, segment_count: usize, segment_spacing: f32) -> Self {
        // Calculate a rough size based on segments
        let size = segment_count as f32 * segment_spacing;
        // Placeholder attributes for a snake
        let attributes = CreatureAttributes::new(
            100.0,                // max_energy
            5.0,                  // energy_recovery_rate
            100.0,                // max_satiety
            1.0,                  // metabolic_rate
            DietType::Carnivore,  // diet_type (let's make it a carnivore for now)
            size,                 // size
            vec!["small_fish".to_string(), "worm".to_string()], // prey_tags
            vec!["snake".to_string(), "medium_predator".to_string()], // self_tags
        );

        Self {
            id: 0, // Default ID, will be overwritten in spawn_rapier
            segment_handles: Vec::with_capacity(segment_count),
            joint_handles: Vec::with_capacity(segment_count.saturating_sub(1)),
            segment_radius,
            segment_count,
            segment_spacing,
            wiggle_timer: 0.0, // Initialize timer
            attributes,        // Initialize attributes
            current_state: CreatureState::Wandering, // Start wandering
        }
    }

    // Renamed from spawn, takes Rapier sets as arguments
    pub fn spawn_rapier(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        impulse_joint_set: &mut ImpulseJointSet,
        initial_position: Vector2<f32>,
        creature_id: u128, // Added creature ID
    ) {
        self.id = creature_id; // Store the ID
        self.segment_handles.clear();
        self.joint_handles.clear();

        let mut parent_handle: Option<RigidBodyHandle> = None;

        for i in 0..self.segment_count {
            let segment_x = initial_position.x + i as f32 * self.segment_spacing;
            let segment_y = initial_position.y;

            // Create RigidBody
            let rb = RigidBodyBuilder::dynamic()
                .translation(vector![segment_x, segment_y])
                .linear_damping(10.0) // Increased damping significantly for water resistance
                .angular_damping(5.0) // Increase angular damping
                .sleeping(false) // Disable sleeping for snake segments
                .ccd_enabled(true) // Enable CCD when disabling sleeping
                .build();
            let segment_handle = rigid_body_set.insert(rb);
            self.segment_handles.push(segment_handle);

            // Create Collider
            let collider = ColliderBuilder::ball(self.segment_radius)
                             .restitution(0.2) // Lower restitution for less bounce
                             .density(100.0) // Significantly higher density (closer to water-like mass)
                             .user_data(creature_id) // Set the creature ID here
                             .build();
            collider_set.insert_with_parent(collider, segment_handle, rigid_body_set);

            // Create joint with the previous segment
            if let Some(prev_handle) = parent_handle {
                let joint = RevoluteJointBuilder::new()
                    // Convert vectors to points for anchors
                    .local_anchor1(Point2::new(self.segment_spacing / 2.0, 0.0))
                    .local_anchor2(Point2::new(-self.segment_spacing / 2.0, 0.0))
                    // Add damping and stiffness for a more "soft" feel
                    .motor_velocity(0.0, 0.0) // Target velocity = 0
                    .motor_max_force(100.0) // Limit motor force
                    .motor_model(MotorModel::ForceBased)
                    //.set_contacts_enabled(false) // Maybe disable segment-segment collision?
                    .build();
                // Insert joint into the provided set
                let joint_handle = impulse_joint_set.insert(prev_handle, segment_handle, joint, true);
                self.joint_handles.push(joint_handle);
            }

            parent_handle = Some(segment_handle);
        }
    }

    // Helper function for the wiggle/movement logic
    fn apply_wiggle(
        &mut self,
        dt: f32,
        impulse_joint_set: &mut ImpulseJointSet,
        amplitude_scale: f32, // Scale factor for wiggle intensity
        frequency_scale: f32, // Scale factor for wiggle speed
        energy_cost_scale: f32, // Scale factor for energy cost
    ) {
        self.wiggle_timer += dt * 3.0 * frequency_scale;

        let target_velocity_amplitude = 1.8 * amplitude_scale; // Slightly reduced base amplitude
        let wiggle_frequency = 1.5;
        let motor_force_factor = 4.0; // Reduced from 10.0
        let base_energy_cost_per_rad_per_sec = 0.5;

        let mut total_applied_velocity = 0.0;

        for (i, handle) in self.joint_handles.iter().enumerate() {
            if let Some(joint) = impulse_joint_set.get_mut(*handle) {
                let phase = self.wiggle_timer + (i as f32 / (self.segment_count - 1) as f32) * std::f32::consts::TAU * wiggle_frequency;
                let target_velocity = phase.sin() * target_velocity_amplitude;
                joint.data.set_motor_velocity(JointAxis::AngX, target_velocity, motor_force_factor);
                total_applied_velocity += target_velocity.abs();
            }
        }

        let energy_consumed = total_applied_velocity * base_energy_cost_per_rad_per_sec * energy_cost_scale * dt;
        self.attributes.consume_energy(energy_consumed);
    }

    // Helper function to apply anisotropic drag.
    // Drag Force = -coeff * velocity_component * |velocity_component| * direction_vector
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

impl Creature for Snake {
    fn id(&self) -> u128 {
        self.id
    }

    fn get_rigid_body_handles(&self) -> &[RigidBodyHandle] {
        &self.segment_handles
    }

    fn get_joint_handles(&self) -> &[ImpulseJointHandle] {
        &self.joint_handles
    }

    // Implement required methods
    fn attributes(&self) -> &CreatureAttributes {
        &self.attributes
    }

    fn attributes_mut(&mut self) -> &mut CreatureAttributes {
        &mut self.attributes
    }

    fn drawing_radius(&self) -> f32 {
        self.segment_radius
    }

    fn type_name(&self) -> &'static str {
        "Snake"
    }

    fn current_state(&self) -> CreatureState {
        self.current_state
    }

    fn update_state_and_behavior(
        &mut self,
        dt: f32,
        _own_id: u128, // Parameter added to match trait, underscore if not used yet
        _rigid_body_set: &mut RigidBodySet, 
        impulse_joint_set: &mut ImpulseJointSet,
        _collider_set: &ColliderSet, // Parameter added
        _query_pipeline: &QueryPipeline, // Parameter added
        _all_creatures_info: &Vec<CreatureInfo>, // Parameter added
        _world_context: &WorldContext, 
    ) {
        // --- State Transition Logic --- 
        let mut next_state = self.current_state; // Start with current state
        let current_energy = self.attributes.energy;

        // Priorities: Fleeing > SeekingFood > Resting > Wandering > Idle 
        // (We only have Resting and Wandering/Idle logic for now)

        if self.attributes.is_tired() {
            next_state = CreatureState::Resting;
        } else if self.attributes.is_hungry() {
             // TODO: Add sensing check here. If food nearby, switch to SeekingFood
             // For now, just keep wandering even if hungry, until we have sensing.
             if self.current_state == CreatureState::Resting { 
                 // If rested enough, start wandering again
                 if self.attributes.energy > self.attributes.max_energy * 0.5 { // Example threshold to stop resting
                     next_state = CreatureState::Wandering;
                 }
             } else { // If not resting, default to wandering
                 next_state = CreatureState::Wandering;
             }
        } else { // Not tired, not hungry
             if self.current_state == CreatureState::Resting { 
                 // If rested enough, start wandering again
                 if self.attributes.energy > self.attributes.max_energy * 0.8 { // Higher threshold to stop resting if not hungry
                     next_state = CreatureState::Wandering;
                 }
             } else { // If not resting, default to wandering
                 next_state = CreatureState::Wandering;
             }
        }
        // TODO: Add transition logic for Fleeing based on sensed predators
        
        if next_state != self.current_state {
            println!(
                "State Transition: {:?} -> {:?} (Energy: {:.2})", 
                self.current_state, next_state, current_energy
            );
        }
        self.current_state = next_state;

        // --- Execute Behavior based on State --- 
        // Log current state and energy before acting
        // println!("Executing State: {:?} (Energy: {:.2})", self.current_state, self.attributes.energy);

        match self.current_state {
            CreatureState::Idle => {
                // Minimal movement or stop motors completely
                self.apply_wiggle(dt, impulse_joint_set, 0.1, 0.5, 0.1); // Very slow, low cost
            }
            CreatureState::Wandering => {
                // Standard wiggle - Increased frequency scale
                self.apply_wiggle(dt, impulse_joint_set, 1.5, 1.5, 1.5); // Increased frequency_scale (1.0->1.5)
            }
            CreatureState::Resting => {
                // No active movement, energy recovery happens passively in App::update
                 // Ensure motors target zero velocity, but keep force available
                 let motor_force_factor = 4.0; // Get the same factor used in apply_wiggle
                 for handle in self.joint_handles.iter() {
                     if let Some(joint) = impulse_joint_set.get_mut(*handle) {
                         // Set target velocity to 0, but KEEP the force factor
                         joint.data.set_motor_velocity(JointAxis::AngX, 0.0, motor_force_factor);
                     }
                 }
            }
            CreatureState::SeekingFood => {
                // TODO: Implement movement towards food
                self.apply_wiggle(dt, impulse_joint_set, 1.0, 1.2, 1.0); 
            }
            CreatureState::Fleeing => {
                // TODO: Implement movement away from predator
                self.apply_wiggle(dt, impulse_joint_set, 1.5, 1.5, 1.5); 
            }
        }
    }

    /// Override the default apply_custom_forces for Snake.
    fn apply_custom_forces(&self, rigid_body_set: &mut RigidBodySet, _world_context: &WorldContext) {
        // --- Apply Hydrodynamic Forces --- 
        // These coefficients NEED TUNING!
        let perp_drag = 5.0;  // Significantly higher drag for sideways motion
        let forward_drag = 0.5; // Lower drag for forward/backward motion

        for handle in self.get_rigid_body_handles() { 
            // Call the associated helper function (now part of impl Snake)
            Snake::apply_anisotropic_drag(*handle, rigid_body_set, perp_drag, forward_drag);
        }
    }

    /// Draws the snake using egui.
    fn draw(
        &self,
        painter: &egui::Painter,
        rigid_body_set: &RigidBodySet,
        world_to_screen: &dyn Fn(Vector2<f32>) -> egui::Pos2,
        zoom: f32,
        is_hovered: bool,
        pixels_per_meter: f32, // Added parameter
    ) {
        let base_color = match self.current_state() {
            CreatureState::Idle => egui::Color32::from_rgb(100, 100, 200), // Bluish
            CreatureState::Wandering => egui::Color32::from_rgb(100, 200, 100), // Greenish
            CreatureState::Resting => egui::Color32::from_rgb(200, 200, 100), // Yellowish
            CreatureState::SeekingFood => egui::Color32::from_rgb(200, 100, 100), // Reddish
            CreatureState::Fleeing => egui::Color32::from_rgb(255, 0, 255),   // Magenta
        };

        let screen_radius = self.drawing_radius() * pixels_per_meter * zoom; // Use passed parameter

        // Get body handles
        let handles = self.get_rigid_body_handles();
        if handles.len() < 2 {
            // Fallback: Draw circles if not enough segments for skin
            for handle in handles {
                if let Some(body) = rigid_body_set.get(*handle) {
                    let pos = body.translation();
                    let screen_pos = world_to_screen(Vector2::new(pos.x, pos.y));

                    if is_hovered {
                        painter.circle_filled(
                            screen_pos,
                            screen_radius * 1.2,
                            egui::Color32::WHITE,
                        );
                    }
                    painter.circle_filled(screen_pos, screen_radius, base_color);
                }
            }
            return; // Exit early
        }

        // --- Draw Snake Skin ---
        let mut world_positions: Vec<Vector2<f32>> = Vec::with_capacity(handles.len());
        for handle in handles {
            if let Some(body) = rigid_body_set.get(*handle) {
                world_positions.push(*body.translation());
            } else {
                world_positions.clear(); // Don't draw partial snake if a body is missing
                break;
            }
        }

        if world_positions.len() < 2 { return; } // Should be redundant due to check above, but safe.

        let mut side1_points: Vec<Vector2<f32>> = Vec::with_capacity(handles.len());
        let mut side2_points: Vec<Vector2<f32>> = Vec::with_capacity(handles.len());
        let radius = self.drawing_radius();

        // Calculate offset points
        for i in 0..world_positions.len() {
            let p_curr = world_positions[i];
            let direction = if i == 0 {
                (world_positions[1] - p_curr).try_normalize(1e-6).unwrap_or_else(Vector2::zeros)
            } else if i == world_positions.len() - 1 {
                 (p_curr - world_positions[i-1]).try_normalize(1e-6).unwrap_or_else(Vector2::zeros)
            } else {
                ((world_positions[i+1] - world_positions[i-1]) / 2.0).try_normalize(1e-6).unwrap_or_else(|| {
                    (world_positions[i+1] - p_curr).try_normalize(1e-6).unwrap_or_else(Vector2::zeros)
                })
            };
            let perpendicular = Vector2::new(-direction.y, direction.x);
            side1_points.push(p_curr + perpendicular * radius);
            side2_points.push(p_curr - perpendicular * radius);
        }

        // Draw skin as individual quadrilaterals
        for i in 0..(world_positions.len() - 1) {
            let quad_world = [
                side1_points[i],
                side1_points[i+1],
                side2_points[i+1],
                side2_points[i],
            ];

            let quad_screen: Vec<egui::Pos2> = quad_world
                .into_iter()
                .map(|wp| world_to_screen(wp))
                .collect();

            if quad_screen.len() == 4 { // Ensure we have 4 points
                if is_hovered {
                    // Draw highlight outline for this segment
                    painter.add(egui::Shape::convex_polygon(
                        quad_screen.clone(),
                        egui::Color32::TRANSPARENT,
                        egui::Stroke::new(screen_radius * 0.4, egui::Color32::WHITE),
                    ));
                }
                // Draw the main skin segment
                painter.add(egui::Shape::convex_polygon(
                    quad_screen,
                    base_color,
                    egui::Stroke::NONE,
                ));
            }
        }
    }
} 