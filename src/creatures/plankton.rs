use rapier2d::prelude::*;
use nalgebra::{Vector2, Point2};
use eframe::egui; // Keep for draw method later
use rand::Rng;

use crate::creature::{Creature, CreatureState, WorldContext, CreatureInfo};
use crate::creature_attributes::{CreatureAttributes, DietType};

/// Simplified info for boid calculation
#[derive(Debug, Clone, Copy)]
pub struct BoidNeighborInfo {
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
}

/// Calculates the combined boid steering impulse.
pub fn calculate_boid_steering_impulse(
    self_position: Vector2<f32>,
    // self_velocity: Vector2<f32>, // Not directly used in current impulse-based boids, but could be for target velocity approaches
    neighbors_info: &[BoidNeighborInfo],
    _perception_radius: f32, // Prefixed with underscore
    separation_distance: f32,
    cohesion_strength: f32,
    separation_strength: f32,
    alignment_strength: f32,
) -> Vector2<f32> {
    let mut separation_force_accumulator = Vector2::zeros();
    let mut alignment_velocity_accumulator = Vector2::zeros();
    let mut cohesion_position_accumulator = Vector2::zeros();
    let local_flockmates_count = neighbors_info.len();

    if local_flockmates_count == 0 {
        return Vector2::zeros();
    }

    for neighbor in neighbors_info {
        cohesion_position_accumulator += neighbor.position;
        alignment_velocity_accumulator += neighbor.velocity;

        let distance = (neighbor.position - self_position).norm();
        if distance < separation_distance && distance > 0.0 { 
            let away_vector = (self_position - neighbor.position).normalize(); // direction from neighbor to self
            separation_force_accumulator += away_vector / distance; 
        }
    }

    let mut boid_impulse = Vector2::zeros();

    // Cohesion
    let cohesion_target = cohesion_position_accumulator / (local_flockmates_count as f32);
    let cohesion_force = (cohesion_target - self_position).try_normalize(1e-6).unwrap_or_else(Vector2::zeros) * cohesion_strength;
    boid_impulse += cohesion_force;

    // Alignment
    let alignment_target_velocity = alignment_velocity_accumulator / (local_flockmates_count as f32);
    let alignment_force = (alignment_target_velocity.try_normalize(1e-6).unwrap_or_else(Vector2::zeros)) * alignment_strength;
    boid_impulse += alignment_force;

    // Separation
    if separation_force_accumulator.norm_squared() > 0.0 { // Only apply if there was a separation candidate
        let separation_force = separation_force_accumulator.normalize() * separation_strength;
        boid_impulse += separation_force;
    }
    
    // The final impulse can be quite strong; consider clamping or scaling if needed, or applying as a force over dt.
    // For now, returning raw impulse sum.
    boid_impulse
}

#[cfg(test)]
mod tests {
    use super::*; // Import items from the parent module (plankton.rs)
    use nalgebra::Vector2;

    const DEFAULT_PERCEPTION_RADIUS: f32 = 10.0;
    const DEFAULT_SEPARATION_DISTANCE: f32 = 2.0;
    const DEFAULT_COHESION_STRENGTH: f32 = 0.1;
    const DEFAULT_SEPARATION_STRENGTH: f32 = 0.2;
    const DEFAULT_ALIGNMENT_STRENGTH: f32 = 0.05;

    // Helper to compare float vectors with a tolerance
    fn assert_vec_approx_eq(a: Vector2<f32>, b: Vector2<f32>, epsilon: f32) {
        assert!((a.x - b.x).abs() < epsilon, "x component mismatch: {} vs {}", a.x, b.x);
        assert!((a.y - b.y).abs() < epsilon, "y component mismatch: {} vs {}", a.y, b.y);
    }

    #[test]
    fn test_boids_no_neighbors() {
        let self_pos = Vector2::new(0.0, 0.0);
        let neighbors = [];
        let impulse = calculate_boid_steering_impulse(
            self_pos, 
            &neighbors, 
            DEFAULT_PERCEPTION_RADIUS, 
            DEFAULT_SEPARATION_DISTANCE, 
            DEFAULT_COHESION_STRENGTH, 
            DEFAULT_SEPARATION_STRENGTH, 
            DEFAULT_ALIGNMENT_STRENGTH
        );
        assert_vec_approx_eq(impulse, Vector2::zeros(), 1e-6);
    }

    #[test]
    fn test_boids_one_neighbor_cohesion() {
        let self_pos = Vector2::new(0.0, 0.0);
        let neighbors = [BoidNeighborInfo { position: Vector2::new(5.0, 0.0), velocity: Vector2::zeros() }];
        // With only cohesion, alignment=0, separation=0
        let impulse = calculate_boid_steering_impulse(
            self_pos, 
            &neighbors, 
            DEFAULT_PERCEPTION_RADIUS, 
            100.0, // Ensure no separation
            1.0,   // Strong cohesion
            0.0,   // No separation
            0.0    // No alignment
        );
        // Should move towards the neighbor (positive x)
        assert!(impulse.x > 0.0, "Cohesion impulse should be positive X");
        assert_vec_approx_eq(impulse, Vector2::new(1.0, 0.0), 1e-6); // Cohesion target is (5,0), dir (1,0) * strength 1.0
    }

    #[test]
    fn test_boids_one_neighbor_alignment() {
        let self_pos = Vector2::new(0.0, 0.0);
        let neighbors = [BoidNeighborInfo { position: Vector2::new(5.0, 0.0), velocity: Vector2::new(0.0, 1.0) }];
        // With only alignment
        let impulse = calculate_boid_steering_impulse(
            self_pos, 
            &neighbors, 
            DEFAULT_PERCEPTION_RADIUS, 
            100.0, // Ensure no separation
            0.0,   // No cohesion
            0.0,   // No separation
            1.0    // Strong alignment
        );
        // Should align with neighbor's velocity (positive y)
        assert!(impulse.y > 0.0, "Alignment impulse should be positive Y");
        assert_vec_approx_eq(impulse, Vector2::new(0.0, 1.0), 1e-6); // Align target is (0,1), dir (0,1) * strength 1.0
    }

    #[test]
    fn test_boids_one_neighbor_separation_too_close() {
        let self_pos = Vector2::new(0.0, 0.0);
        let neighbor_pos = Vector2::new(1.0, 0.0); // Within separation distance of 2.0
        let neighbors = [BoidNeighborInfo { position: neighbor_pos, velocity: Vector2::zeros() }];
        let impulse = calculate_boid_steering_impulse(
            self_pos, 
            &neighbors, 
            DEFAULT_PERCEPTION_RADIUS, 
            DEFAULT_SEPARATION_DISTANCE, // 2.0
            0.0,   // No cohesion
            1.0,   // Strong separation
            0.0    // No alignment
        );
        // Should move away from the neighbor (negative x)
        assert!(impulse.x < 0.0, "Separation impulse should be negative X");
        // Expected direction is (-1,0). Strength is 1.0. 
        // The separation accumulator would be (-1,0)/distance = (-1,0)/1.0 = (-1,0)
        // Normalized (-1,0) * strength 1.0 = (-1.0, 0.0)
        assert_vec_approx_eq(impulse, Vector2::new(-1.0, 0.0), 1e-6);
    }

    #[test]
    fn test_boids_one_neighbor_separation_far_enough() {
        let self_pos = Vector2::new(0.0, 0.0);
        let neighbor_pos = Vector2::new(3.0, 0.0); // Outside separation distance of 2.0
        let neighbors = [BoidNeighborInfo { position: neighbor_pos, velocity: Vector2::zeros() }];
        let impulse = calculate_boid_steering_impulse(
            self_pos, 
            &neighbors, 
            DEFAULT_PERCEPTION_RADIUS, 
            DEFAULT_SEPARATION_DISTANCE, // 2.0
            0.0,   // No cohesion
            1.0,   // Strong separation
            0.0    // No alignment
        );
        // No separation force should be applied if neighbor is far enough
        assert_vec_approx_eq(impulse, Vector2::zeros(), 1e-6);
    }

     #[test]
    fn test_boids_two_neighbors_balanced_cohesion() {
        let self_pos = Vector2::new(0.0, 0.0);
        let neighbors = [
            BoidNeighborInfo { position: Vector2::new(5.0, 0.0), velocity: Vector2::zeros() },
            BoidNeighborInfo { position: Vector2::new(-5.0, 0.0), velocity: Vector2::zeros() },
        ];
        let impulse = calculate_boid_steering_impulse(
            self_pos, 
            &neighbors, 
            DEFAULT_PERCEPTION_RADIUS, 
            100.0, // Ensure no separation
            1.0,   // Strong cohesion
            0.0,   // No separation
            0.0    // No alignment
        );
        // Cohesion target is (0,0), so impulse should be zero
        assert_vec_approx_eq(impulse, Vector2::zeros(), 1e-6);
    }

    #[test]
    fn test_boids_two_neighbors_offset_cohesion_alignment() {
        let self_pos = Vector2::new(0.0, 0.0);
        let neighbors = [
            BoidNeighborInfo { position: Vector2::new(2.0, 1.0), velocity: Vector2::new(1.0, 0.0) },
            BoidNeighborInfo { position: Vector2::new(2.0, -1.0), velocity: Vector2::new(1.0, 0.0) },
        ];
        // Using default strengths, separation distance large enough not to trigger.
        let impulse = calculate_boid_steering_impulse(
            self_pos, 
            &neighbors, 
            DEFAULT_PERCEPTION_RADIUS, 
            1.0, // Separation distance small enough not to trigger for these positions
            DEFAULT_COHESION_STRENGTH, 
            DEFAULT_SEPARATION_STRENGTH, 
            DEFAULT_ALIGNMENT_STRENGTH
        );
        // Cohesion: target is (2.0, 0.0). Normalized dir (1.0, 0.0). Force = (1,0) * 0.1 = (0.1, 0.0)
        // Alignment: target vel is (1.0, 0.0). Normalized dir (1.0, 0.0). Force = (1,0) * 0.05 = (0.05, 0.0)
        // Total expected: (0.15, 0.0)
        assert_vec_approx_eq(impulse, Vector2::new(0.15, 0.0), 1e-6);
    }
}

pub struct Plankton {
    id: u128,
    segment_handles: Vec<RigidBodyHandle>, // Changed from single handle
    joint_handle: Option<ImpulseJointHandle>, // Added joint handle
    attributes: CreatureAttributes,
    current_state: CreatureState,
    pub primary_radius: f32, // Renamed from radius
    pub secondary_radius: f32, // Added second radius
}

#[allow(dead_code)]
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
            id: 0,
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
        self.id = creature_id;
        self.segment_handles.clear();
        self.joint_handle = None;

        let segment_distance = (self.primary_radius + self.secondary_radius) * 0.8; // How far apart segments start

        // --- Create Primary Segment --- 
        let rb1 = RigidBodyBuilder::dynamic()
            .translation(initial_position)
            .linear_damping(20.0)
            .angular_damping(10.0)
            .gravity_scale(1.0)
            .ccd_enabled(true)
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
            .linear_damping(20.0)
            .angular_damping(10.0)
            .gravity_scale(1.0)
            .ccd_enabled(true)
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
        world_context: &WorldContext, 
    ) {
        // Constants for controlling net vertical acceleration (relative to world gravity magnitude of 1.0)
        const BASE_BUOYANCY_FORCE: f32 = 0.002;  // Base force magnitude
        const NET_GRAVITY_ACCEL_SCALE_SEEKING_FOOD_LOW: f32 = 0.02;    
        const NET_GRAVITY_ACCEL_SCALE_SEEKING_FOOD_HIGH: f32 = -0.2;   
        const NET_GRAVITY_ACCEL_SCALE_SEEKING_FOOD_INZONE: f32 = 0.0;  
        const NET_GRAVITY_ACCEL_SCALE_WANDERING: f32 = -0.05;         
        const NET_GRAVITY_ACCEL_SCALE_RESTING: f32 = -0.1;            

        // Add oscillation parameters
        const OSCILLATION_AMPLITUDE: f32 = 0.05;  
        const OSCILLATION_FREQUENCY: f32 = 0.3;   

        // Velocity control parameters
        const MAX_VERTICAL_SPEED: f32 = 0.5;  // Maximum vertical speed
        const VERTICAL_DAMPING: f32 = 0.1;    // Damping factor for vertical movement
        const HORIZONTAL_DAMPING: f32 = 0.05; // Damping factor for horizontal movement

        let light_zone_target_min_y = world_context.world_height * 0.05;
        let light_zone_target_max_y = world_context.world_height * 0.35;

        for handle in &self.segment_handles {
            if let Some(body) = rigid_body_set.get_mut(*handle) {
                let current_y = body.translation().y;
                let current_x = body.translation().x;
                let current_velocity = *body.linvel();
                
                // Calculate oscillation based on x position to create a wave-like pattern
                let oscillation = (current_x * OSCILLATION_FREQUENCY).sin() * OSCILLATION_AMPLITUDE;
                
                let target_net_accel_y_factor = match self.current_state {
                    CreatureState::SeekingFood => {
                        if current_y < light_zone_target_min_y {
                            NET_GRAVITY_ACCEL_SCALE_SEEKING_FOOD_LOW
                        } else if current_y > light_zone_target_max_y {
                            NET_GRAVITY_ACCEL_SCALE_SEEKING_FOOD_HIGH
                        } else {
                            NET_GRAVITY_ACCEL_SCALE_SEEKING_FOOD_INZONE
                        }
                    }
                    CreatureState::Wandering | CreatureState::Idle => {
                        NET_GRAVITY_ACCEL_SCALE_WANDERING + oscillation
                    }
                    CreatureState::Resting => {
                        NET_GRAVITY_ACCEL_SCALE_RESTING + oscillation * 0.5
                    }
                    CreatureState::Fleeing => {
                        NET_GRAVITY_ACCEL_SCALE_WANDERING + oscillation
                    }
                };

                // Calculate base buoyancy force
                let buoyancy_force_y = BASE_BUOYANCY_FORCE * (1.0 + target_net_accel_y_factor);
                
                // Apply velocity-dependent damping
                let mut final_force_y = buoyancy_force_y;
                
                // Vertical velocity damping
                if current_velocity.y.abs() > MAX_VERTICAL_SPEED {
                    // If exceeding max speed, apply strong damping
                    final_force_y -= current_velocity.y * VERTICAL_DAMPING * 2.0;
                } else {
                    // Normal damping
                    final_force_y -= current_velocity.y * VERTICAL_DAMPING;
                }
                
                // Horizontal velocity damping
                let damping_force_x = -current_velocity.x * HORIZONTAL_DAMPING;
                
                // // Debug logging for every 10th frame (roughly 6 times per second at 60fps)
                // if self.id == 10 && self.id % 10 == 0 {  // Only log for plankton with ID 10
                //     tracing::debug!(
                //         plankton_id = self.id,
                //         state = ?self.current_state,
                //         position = ?(current_x, current_y),
                //         velocity = ?(current_velocity.x, current_velocity.y),
                //         velocity_magnitude = current_velocity.norm(),
                //         mass = body.mass(),
                //         oscillation = oscillation,
                //         target_accel_factor = target_net_accel_y_factor,
                //         buoyancy_force = buoyancy_force_y,
                //         final_force = final_force_y,
                //         damping_force_x = damping_force_x,
                //         energy = self.attributes.energy,
                //         max_energy = self.attributes.max_energy,
                //         "Plankton debug info"
                //     );

                //     // Add warning if velocity is too high
                //     if current_velocity.norm() > 3.0 {
                //         tracing::warn!(
                //             plankton_id = self.id,
                //             velocity = ?(current_velocity.x, current_velocity.y),
                //             velocity_magnitude = current_velocity.norm(),
                //             "Plankton moving too fast!"
                //         );
                //     }
                // }
                
                // Add velocity damping if moving too fast
                if current_velocity.norm() > 2.0 {
                    body.set_linear_damping(20.0);
                } else {
                    body.set_linear_damping(12.0);
                }
                
                // Apply the final forces
                body.add_force(Vector2::new(damping_force_x, final_force_y), true);
            }
        }
    }
}

impl Creature for Plankton {
    fn id(&self) -> u128 {
        self.id
    }

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
        dt: f32,
        own_id: u128,
        rigid_body_set: &mut RigidBodySet,
        _impulse_joint_set: &mut ImpulseJointSet,
        collider_set: &ColliderSet,
        query_pipeline: &QueryPipeline,
        all_creatures_info: &Vec<CreatureInfo>,
        world_context: &WorldContext,
    ) {
        // Boids parameters (can be tuned)
        let perception_radius: f32 = self.primary_radius * 10.0;  // Reduced from 15.0
        let separation_distance: f32 = self.primary_radius * 1.5;  // Reduced from 2.0
        let cohesion_strength: f32 = 0.15;   // Reduced from 0.2
        let separation_strength: f32 = 0.25;  // Reduced from 0.3
        let alignment_strength: f32 = 0.1;    // Reduced from 0.15

        let self_primary_handle = self.segment_handles.get(0).cloned().unwrap_or_else(RigidBodyHandle::invalid);
        let self_position = rigid_body_set.get(self_primary_handle).map_or(Vector2::zeros(), |b| *b.translation());
        let _self_velocity = rigid_body_set.get(self_primary_handle).map_or(Vector2::zeros(), |b| *b.linvel());

        // --- Sensing Phase using QueryPipeline --- 
        let mut boid_neighbors: Vec<BoidNeighborInfo> = Vec::new();
        let perception_shape = Ball::new(perception_radius);
        let perception_shape_pos = Isometry::new(self_position, 0.0);
        
        // Modified filter to include all creatures
        let interaction_filter = InteractionGroups::new(Group::GROUP_1, Group::GROUP_1);
        let query_filter = QueryFilter::new()
            .groups(interaction_filter)
            .exclude_rigid_body(self_primary_handle);

        query_pipeline.intersections_with_shape(
            rigid_body_set,
            collider_set,
            &perception_shape_pos,
            &perception_shape,
            query_filter,
            |intersecting_collider_handle| {
                let intersecting_collider = match collider_set.get(intersecting_collider_handle) {
                    Some(c) => c,
                    None => return true,
                };

                let creature_id_from_collider = intersecting_collider.user_data;
                if creature_id_from_collider == u128::MAX { return true; } // Skip walls
                if creature_id_from_collider == own_id { return true; } // Skip self

                // Find this creature in all_creatures_info
                if let Some(other_creature_info) = all_creatures_info.iter().find(|info| info.id == creature_id_from_collider) {
                    if other_creature_info.creature_type_name == "Plankton" {
                        // Only add if within perception radius
                        let distance = (other_creature_info.position - self_position).norm();
                        if distance <= perception_radius {
                            boid_neighbors.push(BoidNeighborInfo {
                                position: other_creature_info.position,
                                velocity: other_creature_info.velocity,
                            });
                        }
                    }
                }
                true
            },
        );

        // Calculate Boid Impulse
        let boid_impulse = calculate_boid_steering_impulse(
            self_position,
            &boid_neighbors,
            perception_radius,
            separation_distance,
            cohesion_strength,
            separation_strength,
            alignment_strength
        );

        // // Debug logging for boids behavior
        // if self.id == 10 && self.id % 10 == 0 {  // Only log for plankton with ID 10
        //     tracing::debug!(
        //         plankton_id = self.id,
        //         neighbors = boid_neighbors.len(),
        //         boid_impulse = ?(boid_impulse.x, boid_impulse.y),
        //         boid_impulse_magnitude = boid_impulse.norm(),
        //         self_velocity = ?(self_velocity.x, self_velocity.y),
        //         self_velocity_magnitude = self_velocity.norm(),
        //         "Plankton boids debug info"
        //     );
        // }

        // State transition logic - use primary segment for position check
        let current_y = self_position.y;

        // Define energy thresholds for state changes
        let energy_critically_low_threshold = self.attributes.max_energy * 0.21; // Changed from 0.25 
        let energy_comfortable_threshold = self.attributes.max_energy * 0.65; 

        // Define the "light zone" for SeekingFood behavior reference
        let light_zone_ideal_min_y = world_context.world_height * 0.1; 
        let light_zone_ideal_max_y = world_context.world_height * 0.45; // Slightly below absolute ceiling for safety

        let mut next_state = self.current_state;

        if self.attributes.is_tired() { 
            next_state = CreatureState::Resting;
        } else {
            match self.current_state {
                CreatureState::Resting => {
                    if self.attributes.energy >= energy_comfortable_threshold {
                        next_state = CreatureState::Wandering; 
                    }
                }
                CreatureState::Wandering => {
                    if self.attributes.energy < energy_critically_low_threshold {
                        next_state = CreatureState::SeekingFood; 
                    }
                }
                CreatureState::SeekingFood => {
                    if self.attributes.energy >= energy_comfortable_threshold {
                         // Only switch to wandering if energy is high AND they are somewhat in a good spot
                         // This prevents them from immediately leaving the light zone if they just arrived.
                        if current_y >= light_zone_ideal_min_y {
                            next_state = CreatureState::Wandering;
                        }
                    }
                }
                CreatureState::Idle | CreatureState::Fleeing => { 
                    if self.attributes.energy < energy_critically_low_threshold {
                        next_state = CreatureState::SeekingFood;
                    } else {
                        next_state = CreatureState::Wandering;
                    }
                }
            }
        }
        self.current_state = next_state;


        // --- Execute Behavior based on State --- 
        match self.current_state {
            CreatureState::Wandering => {
                if let Some(body) = rigid_body_set.get_mut(self_primary_handle) {
                    if self_primary_handle != RigidBodyHandle::invalid() { 
                        let mut rng = rand::thread_rng();
                        let impulse_strength = 0.05; // Increased from 0.02
                        let random_impulse = Vector2::new(
                            rng.gen_range(-impulse_strength..impulse_strength),
                            rng.gen_range(-impulse_strength..impulse_strength)
                        );
                        // Apply boid impulses along with random wandering
                        body.apply_impulse(random_impulse + boid_impulse, true);
                    }
                 }
            }
            CreatureState::SeekingFood => { 
                // Energy recovery for plankton happens here if in light zone
                let energy_cap_for_photosynthesis = self.attributes.max_energy * 0.9;
                if current_y >= light_zone_ideal_min_y && current_y <= light_zone_ideal_max_y && self.attributes.energy < energy_cap_for_photosynthesis {
                    self.attributes.energy = (self.attributes.energy + self.attributes.energy_recovery_rate * dt).min(self.attributes.max_energy);
                }
                // Buoyancy handles upward movement if needed (defined in apply_buoyancy_and_drag)
            }
            CreatureState::Resting => { /* Buoyancy handles sinking */ }
            CreatureState::Idle => { /* Do nothing */}
            CreatureState::Fleeing => { /* Do nothing */}
        }
    }

    fn apply_custom_forces(&self, rigid_body_set: &mut RigidBodySet, world_context: &WorldContext) {
        // Call the helper method, now passing world_context
        self.apply_buoyancy_and_drag(rigid_body_set, world_context);
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
