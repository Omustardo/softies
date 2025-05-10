use rapier2d::prelude::*;
use nalgebra::{Point2, Vector2};
use eframe::egui; // Add egui import
use rand::{self, Rng}; // Add Rng trait import

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
    rest_timer: f32,   // Timer to track rest time
    attributes: CreatureAttributes, // Added attributes field
    current_state: CreatureState, // Added state field
    // Add new fields for target tracking
    target_position: Option<Vector2<f32>>,
    target_update_timer: f32,
    last_position: Vector2<f32>,
    stuck_timer: f32,
    // Add debug fields
    debug_info: DebugInfo,
}

#[derive(Default)]
struct DebugInfo {
    max_velocity: f32,
    collision_count: u32,
    last_collision_time: f32,
    problematic_segments: Vec<usize>,
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

        // Initialize rest_timer with a random value between 0 and 5 seconds
        let mut rng = rand::thread_rng();
        let rest_timer = rng.gen_range(0.0..5.0);

        Self {
            id: 0, // Default ID, will be overwritten in spawn_rapier
            segment_handles: Vec::with_capacity(segment_count),
            joint_handles: Vec::with_capacity(segment_count.saturating_sub(1)),
            segment_radius,
            segment_count,
            segment_spacing,
            wiggle_timer: 0.0, // Initialize timer
            rest_timer,        // Initialize with random value
            attributes,        // Initialize attributes
            current_state: CreatureState::Wandering, // Start wandering
            target_position: None,
            target_update_timer: 0.0,
            last_position: Vector2::zeros(),
            stuck_timer: 0.0,
            debug_info: DebugInfo::default(),
        }
    }

    // Renamed from spawn, takes Rapier sets as arguments
    pub fn spawn_rapier(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        impulse_joint_set: &mut ImpulseJointSet,
        initial_position: Vector2<f32>,
        creature_id: u128,
    ) {
        self.id = creature_id;
        self.segment_handles.clear();
        self.joint_handles.clear();

        let mut parent_handle: Option<RigidBodyHandle> = None;
        let mut rng = rand::thread_rng();
        
        let initial_angle: f32 = rng.gen_range(-0.02..0.02); // Moderate angle range
        
        for i in 0..self.segment_count {
            let segment_x = initial_position.x + (i as f32) * self.segment_spacing * initial_angle.cos();
            let segment_y = initial_position.y + (i as f32) * self.segment_spacing * initial_angle.sin();
            let orientation = initial_angle;

            // Create RigidBody with moderate damping
            let rb = RigidBodyBuilder::dynamic()
                .translation(vector![segment_x, segment_y])
                .rotation(orientation)
                .linear_damping(15.0) // Moderate damping
                .angular_damping(8.0)  // Moderate damping
                .build();
            let segment_handle = rigid_body_set.insert(rb);
            self.segment_handles.push(segment_handle);

            // Create Collider with moderate parameters
            let collider = ColliderBuilder::ball(self.segment_radius)
                .restitution(0.0)  // No bounce
                .density(3.0)      // Moderate density
                .friction(0.1)     // Moderate friction
                .user_data(creature_id)
                .build();
            collider_set.insert_with_parent(collider, segment_handle, rigid_body_set);

            // Create joint with moderate parameters
            if let Some(prev_handle) = parent_handle {
                let joint = RevoluteJointBuilder::new()
                    .local_anchor1(Point2::new(self.segment_spacing / 2.0, 0.0))
                    .local_anchor2(Point2::new(-self.segment_spacing / 2.0, 0.0))
                    .motor_velocity(0.0, 0.0)
                    .motor_max_force(0.3)  // Moderate force
                    .motor_model(MotorModel::ForceBased)
                    .limits([-0.02, 0.02])   // Moderate limits
                    .build();
                let joint_handle = impulse_joint_set.insert(prev_handle, segment_handle, joint, true);
                self.joint_handles.push(joint_handle);
            }

            parent_handle = Some(segment_handle);
        }
    }

    // Add new method to update target position
    fn update_target_position(&mut self, _rigid_body_set: &RigidBodySet, world_context: &WorldContext) {
        let mut rng = rand::thread_rng();
        
        // Update target every 3-5 seconds or if we're stuck
        if self.target_position.is_none() || self.target_update_timer > rng.gen_range(3.0..5.0) || self.stuck_timer > 1.0 {
            // Generate new target within world bounds
            // Use world_height and assume square world for now
            let world_size = world_context.world_height;
            let new_target = Vector2::new(
                rng.gen_range(-world_size/2.0..world_size/2.0),
                rng.gen_range(-world_size/2.0..world_size/2.0)
            );
            self.target_position = Some(new_target);
            self.target_update_timer = 0.0;
            self.stuck_timer = 0.0;
        }
    }

    // Add method to check if snake is stuck
    fn check_if_stuck(&mut self, rigid_body_set: &RigidBodySet) {
        if let Some(head_handle) = self.segment_handles.first() {
            if let Some(head_body) = rigid_body_set.get(*head_handle) {
                let current_pos = Vector2::new(head_body.translation().x, head_body.translation().y);
                let distance_moved = (current_pos - self.last_position).norm();
                
                if distance_moved < 0.1 {
                    self.stuck_timer += 0.016; // Assuming 60 FPS
                } else {
                    self.stuck_timer = 0.0;
                }
                
                self.last_position = current_pos;
            }
        }
    }

    // Add method to check for self-collision and problematic states
    fn check_safety(&mut self, rigid_body_set: &RigidBodySet, dt: f32) -> bool {
        let mut is_safe = true;
        self.debug_info.problematic_segments.clear();

        // Get all segment positions
        let mut segment_positions = Vec::new();
        for handle in &self.segment_handles {
            if let Some(body) = rigid_body_set.get(*handle) {
                let pos = Vector2::new(body.translation().x, body.translation().y);
                let vel = body.linvel();
                segment_positions.push((pos, vel));

                // Check velocity bounds - extremely reduced maximum safe speed
                let speed = vel.norm();
                if speed > 5.0 {  // Reduced from 10.0
                    is_safe = false;
                    self.debug_info.max_velocity = speed;
                }
            }
        }

        // Check for self-collision and segment spacing
        for i in 0..segment_positions.len() {
            for j in (i + 2)..segment_positions.len() {
                let (pos1, _) = segment_positions[i];
                let (pos2, _) = segment_positions[j];
                let distance = (pos1 - pos2).norm();
                
                // If segments are too close, mark as problematic
                if distance < self.segment_radius * 2.5 {  // Increased from 2.0
                    is_safe = false;
                    self.debug_info.problematic_segments.push(i);
                    self.debug_info.problematic_segments.push(j);
                    self.debug_info.collision_count += 1;
                    self.debug_info.last_collision_time = 0.0;
                }
            }
        }

        // Update debug timers
        self.debug_info.last_collision_time += dt;

        is_safe
    }

    // Add method to correct problematic states
    fn correct_problematic_state(&mut self, rigid_body_set: &mut RigidBodySet) {
        // If we have problematic segments, try to straighten them out
        if !self.debug_info.problematic_segments.is_empty() {
            // First, collect all the positions we need
            let mut segment_positions = Vec::new();
            for handle in &self.segment_handles {
                if let Some(body) = rigid_body_set.get(*handle) {
                    let pos = Vector2::new(body.translation().x, body.translation().y);
                    segment_positions.push(pos);
                }
            }

            // Then apply corrections
            for &segment_idx in &self.debug_info.problematic_segments {
                if let Some(handle) = self.segment_handles.get(segment_idx) {
                    if let Some(body) = rigid_body_set.get_mut(*handle) {
                        // Apply damping to problematic segments
                        body.set_linvel(vector![0.0, 0.0], true);
                        body.set_angvel(0.0, true);
                        
                        // If it's not the head, try to align with adjacent segments
                        if segment_idx > 0 && segment_idx < self.segment_count - 1 {
                            let prev_pos = segment_positions[segment_idx - 1];
                            let next_pos = segment_positions[segment_idx + 1];
                            let target_pos = (prev_pos + next_pos) * 0.5;
                            
                            // Gently move towards the target position
                            let current_pos = Vector2::new(body.translation().x, body.translation().y);
                            let correction = (target_pos - current_pos) * 0.1;
                            body.set_translation(vector![
                                current_pos.x + correction.x,
                                current_pos.y + correction.y
                            ], true);
                        }
                    }
                }
            }
        }
    }

    // Add method to check if position is within bounds
    fn is_within_bounds(&self, pos: Vector2<f32>, world_context: &WorldContext) -> bool {
        let half_size = world_context.world_height / 2.0;
        let margin = self.segment_radius * 3.0; // Increased margin for better safety
        
        pos.x.abs() < half_size - margin && pos.y.abs() < half_size - margin
    }

    // Add method to get a safe position within bounds
    fn get_safe_position(&self, world_context: &WorldContext) -> Vector2<f32> {
        let half_size = world_context.world_height / 2.0;
        let margin = self.segment_radius * 6.0; // Increased margin for better safety
        let mut rng = rand::thread_rng();
        
        Vector2::new(
            rng.gen_range(-half_size + margin..half_size - margin),
            rng.gen_range(-half_size + margin..half_size - margin)
        )
    }

    // Add method to reset snake to a safe position
    fn reset_to_safe_position(&mut self, rigid_body_set: &mut RigidBodySet, world_context: &WorldContext) {
        let base_pos = self.get_safe_position(world_context);
        let mut rng = rand::thread_rng();
        let initial_angle: f32 = rng.gen_range(-0.01..0.01); // Reduced angle range for more stability

        // Reset each segment to a proper formation with gentle curve
        for (i, handle) in self.segment_handles.iter().enumerate() {
            if let Some(body) = rigid_body_set.get_mut(*handle) {
                // Calculate position in a gentle curve
                let segment_x = base_pos.x + (i as f32) * self.segment_spacing * initial_angle.cos();
                let segment_y = base_pos.y + (i as f32) * self.segment_spacing * initial_angle.sin();
                
                // Reset position and velocity immediately
                body.set_translation(vector![segment_x, segment_y], true);
                body.set_rotation(Rotation::new(initial_angle), true);
                body.set_linvel(vector![0.0, 0.0], true);
                body.set_angvel(0.0, true);
            }
        }

        // Reset timers and state
        self.wiggle_timer = 0.0;
        self.stuck_timer = 0.0;
        self.target_position = None;
        self.target_update_timer = 0.0;
        self.last_position = base_pos;
    }

    // Add method to calculate boundary avoidance force
    fn calculate_boundary_force(&self, pos: Vector2<f32>, world_context: &WorldContext) -> Option<Vector2<f32>> {
        let half_size = world_context.world_height / 2.0;
        let margin = self.segment_radius * 3.0; // Moderate margin
        
        // Calculate distance to each boundary
        let dist_to_right = half_size - pos.x;
        let dist_to_left = half_size + pos.x;
        let dist_to_top = half_size - pos.y;
        let dist_to_bottom = half_size + pos.y;
        
        // If we're too close to any boundary, calculate avoidance force
        if dist_to_right < margin || dist_to_left < margin || dist_to_top < margin || dist_to_bottom < margin {
            let mut force = Vector2::zeros();
            
            // Add force away from each boundary we're too close to
            if dist_to_right < margin {
                force.x -= (margin - dist_to_right) * 5.0; // Moderate force
            }
            if dist_to_left < margin {
                force.x += (margin - dist_to_left) * 5.0;
            }
            if dist_to_top < margin {
                force.y -= (margin - dist_to_top) * 5.0;
            }
            if dist_to_bottom < margin {
                force.y += (margin - dist_to_bottom) * 5.0;
            }
            
            // Normalize and scale the force
            if let Some(normalized) = force.try_normalize(1e-6) {
                return Some(normalized * 15.0); // Moderate force strength
            }
        }
        
        None
    }

    // Add method to clamp position within bounds
    fn clamp_position(&self, pos: Vector2<f32>, world_context: &WorldContext) -> Vector2<f32> {
        let half_size = world_context.world_height / 2.0;
        let margin = self.segment_radius * 3.0; // Increased margin
        
        Vector2::new(
            pos.x.clamp(-half_size + margin, half_size - margin),
            pos.y.clamp(-half_size + margin, half_size - margin)
        )
    }

    // Add method to check and correct all segments
    fn check_and_correct_segments(&mut self, rigid_body_set: &mut RigidBodySet, world_context: &WorldContext) {
        let mut needs_reset = false;
        let half_size = world_context.world_height / 2.0;
        let margin = self.segment_radius * 3.0; // Moderate margin
        
        // Check all segments for boundary violations
        for handle in &self.segment_handles {
            if let Some(body) = rigid_body_set.get_mut(*handle) {
                let pos = Vector2::new(body.translation().x, body.translation().y);
                
                // Check if out of bounds
                if pos.x.abs() >= half_size - margin || pos.y.abs() >= half_size - margin {
                    // Calculate correction force
                    let mut correction = Vector2::zeros();
                    
                    // X-axis correction
                    if pos.x.abs() >= half_size - margin {
                        correction.x = -pos.x.signum() * 20.0; // Moderate correction force
                    }
                    
                    // Y-axis correction
                    if pos.y.abs() >= half_size - margin {
                        correction.y = -pos.y.signum() * 20.0; // Moderate correction force
                    }
                    
                    // Apply correction force
                    body.add_force(correction, true);
                    
                    // Moderate damping when near boundaries
                    let vel = body.linvel();
                    body.set_linvel(vel * 0.8, true); // Moderate velocity reduction
                    
                    // If too close to boundary, mark for reset
                    if pos.x.abs() >= half_size - margin/2.0 || pos.y.abs() >= half_size - margin/2.0 {
                        needs_reset = true;
                    }
                }
            }
        }
        
        if needs_reset {
            self.reset_to_safe_position(rigid_body_set, world_context);
            return;
        }
        
        // Apply boundary forces to all segments
        for handle in &self.segment_handles {
            if let Some(body) = rigid_body_set.get_mut(*handle) {
                let pos = Vector2::new(body.translation().x, body.translation().y);
                if let Some(force) = self.calculate_boundary_force(pos, world_context) {
                    body.add_force(force, true);
                }
            }
        }
    }

    fn apply_wiggle(
        &mut self,
        dt: f32,
        impulse_joint_set: &mut ImpulseJointSet,
        rigid_body_set: &mut RigidBodySet,
        mut amplitude_scale: f32,
        mut frequency_scale: f32,
        energy_cost_scale: f32,
    ) {
        let id_based_phase = (self.id as f32) * 0.1;
        self.wiggle_timer += dt * frequency_scale;

        // Get the head segment's current orientation and position
        if let Some(head_handle) = self.segment_handles.first() {
            if let Some(head_body) = rigid_body_set.get_mut(*head_handle) {
                let head_pos = Vector2::new(head_body.translation().x, head_body.translation().y);
                let head_angle = head_body.rotation().angle();
                
                // Calculate desired direction based on target
                let desired_direction = if let Some(target) = self.target_position {
                    (target - head_pos).try_normalize(1e-6).unwrap_or_else(Vector2::zeros)
                } else {
                    Vector2::new(head_angle.cos(), head_angle.sin())
                };

                // Moderate rotation with maximum angular velocity
                let current_dir = Vector2::new(head_angle.cos(), head_angle.sin());
                let angle_diff = desired_direction.y.atan2(desired_direction.x) - head_angle;
                let clamped_angle = angle_diff.clamp(-0.02, 0.02);  // Moderate angle range
                let max_angular_velocity = 0.3;  // Moderate maximum angular velocity
                let angular_velocity = clamped_angle * 0.1;  // Moderate torque
                head_body.set_angvel(angular_velocity.clamp(-max_angular_velocity, max_angular_velocity), true);

                // Moderate forward force with maximum velocity
                let forward_force = current_dir * 0.2 * amplitude_scale;  // Moderate force
                let current_vel = head_body.linvel();
                let max_velocity = 2.0;  // Moderate maximum linear velocity
                if current_vel.norm() < max_velocity {
                    head_body.add_force(forward_force, true);
                } else {
                    // Apply moderate damping when exceeding max velocity
                    head_body.set_linvel(current_vel * 0.8, true);
                }

                // Moderate wave pattern
                let wave_length = 1.0;
                let wave_amplitude = 0.01 * amplitude_scale;  // Moderate amplitude

                for (i, handle) in self.joint_handles.iter().enumerate() {
                    if let Some(joint) = impulse_joint_set.get_mut(*handle) {
                        let segment_phase = (i as f32) * wave_length;
                        let phase = self.wiggle_timer + segment_phase + id_based_phase;
                        let target_velocity = (phase.sin() * wave_amplitude) * frequency_scale;
                        joint.data.set_motor_velocity(JointAxis::AngX, target_velocity, 0.1);  // Moderate motor force
                    }
                }

                // Apply energy cost based on movement
                let energy_consumed = amplitude_scale * frequency_scale * energy_cost_scale * dt;
                self.attributes.consume_energy(energy_consumed);
            }
        }
    }

    // Helper function to apply anisotropic drag.
    // Drag Force = -coeff * velocity_component * |velocity_component| * direction_vector
    fn apply_anisotropic_drag(
        body_handle: RigidBodyHandle,
        rigid_body_set: &mut RigidBodySet,
        perp_drag_coeff: f32,
        forward_drag_coeff: f32,
    ) {
        if let Some(body) = rigid_body_set.get_mut(body_handle) {
            let linvel = *body.linvel();
            // Ensure velocity is not NaN or infinite, which can cause issues
            if !linvel.x.is_finite() || !linvel.y.is_finite() {
                return;
            }

            let angle = body.rotation().angle();
            let forward_dir = Vector2::new(angle.cos(), angle.sin());
            let right_dir = Vector2::new(-angle.sin(), angle.cos());

            let v_forward = linvel.dot(&forward_dir);
            let v_perpendicular = linvel.dot(&right_dir);

            // Quadratic drag model with increased coefficients
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

    // Add debug drawing
    fn draw_debug_info(
        &self,
        painter: &egui::Painter,
        rigid_body_set: &RigidBodySet,
        world_to_screen: &dyn Fn(Vector2<f32>) -> egui::Pos2,
        zoom: f32,
    ) {
        // Draw problematic segments
        for &segment_idx in &self.debug_info.problematic_segments {
            if let Some(handle) = self.segment_handles.get(segment_idx) {
                if let Some(body) = rigid_body_set.get(*handle) {
                    let pos = Vector2::new(body.translation().x, body.translation().y);
                    let screen_pos = world_to_screen(pos);
                    
                    // Draw red circle around problematic segment
                    painter.circle_stroke(
                        screen_pos,
                        self.segment_radius * 2.0 * zoom,
                        egui::Stroke::new(2.0, egui::Color32::RED),
                    );
                }
            }
        }

        // Draw velocity indicator
        if let Some(head_handle) = self.segment_handles.first() {
            if let Some(head_body) = rigid_body_set.get(*head_handle) {
                let pos = Vector2::new(head_body.translation().x, head_body.translation().y);
                let vel = head_body.linvel();
                let screen_pos = world_to_screen(pos);
                let screen_vel = world_to_screen(pos + vel) - screen_pos;
                
                // Draw velocity vector
                painter.line_segment(
                    [screen_pos, screen_pos + screen_vel],
                    egui::Stroke::new(1.0, egui::Color32::YELLOW),
                );
            }
        }
    }

    // Add method to handle collision events
    fn handle_collision(&mut self, rigid_body_set: &mut RigidBodySet, other_id: u128) {
        // If we collide with another snake, reduce our velocity to prevent glitches
        if let Some(head_handle) = self.segment_handles.first() {
            if let Some(head_body) = rigid_body_set.get_mut(*head_handle) {
                let current_vel = head_body.linvel();
                // Reduce velocity by 50% when colliding with another snake
                head_body.set_linvel(current_vel * 0.5, true);
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
        _own_id: u128,
        rigid_body_set: &mut RigidBodySet,
        impulse_joint_set: &mut ImpulseJointSet,
        _collider_set: &ColliderSet,
        _query_pipeline: &QueryPipeline,
        _all_creatures_info: &Vec<CreatureInfo>,
        world_context: &WorldContext,
    ) {
        // Check and correct all segments for boundary violations
        self.check_and_correct_segments(rigid_body_set, world_context);

        // Update target position and check if stuck
        self.update_target_position(rigid_body_set, world_context);
        self.check_if_stuck(rigid_body_set);
        self.target_update_timer += dt;

        // --- State Transition Logic --- 
        let mut next_state = self.current_state; // Start with current state
        
        // Update rest timer
        if self.current_state == CreatureState::Resting {
            self.rest_timer += dt;
        } else {
            self.rest_timer = 0.0;
        }

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
        
        self.current_state = next_state;

        // --- Execute Behavior based on State --- 
        match self.current_state {
            CreatureState::Idle => {
                self.apply_wiggle(dt, impulse_joint_set, rigid_body_set, 0.1, 0.3, 0.1);
            }
            CreatureState::Wandering => {
                let energy_factor = self.attributes.energy / self.attributes.max_energy;
                let amplitude = 1.0 * energy_factor;
                let frequency = 1.0 * (1.0 + energy_factor * 0.3);
                self.apply_wiggle(dt, impulse_joint_set, rigid_body_set, amplitude, frequency, 1.0);
            }
            CreatureState::Resting => {
                let motor_force_factor = 2.0;
                for handle in self.joint_handles.iter() {
                    if let Some(joint) = impulse_joint_set.get_mut(*handle) {
                        joint.data.set_motor_velocity(JointAxis::AngX, 0.0, motor_force_factor);
                    }
                }
            }
            CreatureState::SeekingFood => {
                let hunger_factor = 1.0 - (self.attributes.energy / self.attributes.max_energy);
                let amplitude = 1.5 * (1.0 + hunger_factor);
                let frequency = 1.5 * (1.0 + hunger_factor * 0.3);
                self.apply_wiggle(dt, impulse_joint_set, rigid_body_set, amplitude, frequency, 1.5);
            }
            CreatureState::Fleeing => {
                self.apply_wiggle(dt, impulse_joint_set, rigid_body_set, 2.0, 1.5, 2.0);
            }
        }
    }

    /// Override the default apply_custom_forces for Snake.
    fn apply_custom_forces(&self, rigid_body_set: &mut RigidBodySet, _world_context: &WorldContext) {
        // Moderate drag coefficients for stability
        let perp_drag = 15.0;  // Moderate drag for sideways motion
        let forward_drag = 5.0; // Moderate drag for forward/backward motion

        for handle in self.get_rigid_body_handles() { 
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

        // Add debug drawing when hovered
        if is_hovered {
            self.draw_debug_info(painter, rigid_body_set, world_to_screen, zoom);
        }
    }
}

// Add a physics hooks implementation to handle collisions
struct SnakePhysicsHooks;

impl PhysicsHooks for SnakePhysicsHooks {
    fn filter_contact_pair(&self, context: &PairFilterContext) -> Option<SolverFlags> {
        // Get the user data (creature IDs) of both colliders
        let id1 = context.colliders[context.collider1].user_data;
        let id2 = context.colliders[context.collider2].user_data;

        // If both colliders are from the same snake, disable contact computation
        if id1 == id2 {
            return None;
        }

        // For collisions between different snakes, enable contact computation but with reduced forces
        Some(SolverFlags::COMPUTE_IMPULSES)
    }

    fn modify_solver_contacts(&self, context: &mut ContactModificationContext) {
        // Get the user data (creature IDs) of both colliders
        let id1 = context.colliders[context.collider1].user_data;
        let id2 = context.colliders[context.collider2].user_data;

        // If this is a collision between different snakes
        if id1 != id2 {
            // Reduce the friction and restitution to prevent sticking and bouncing
            for solver_contact in &mut *context.solver_contacts {
                solver_contact.friction = 0.3;
                solver_contact.restitution = 0.1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector2;
    use std::f32;
    use rapier2d::prelude::*;
    use std::collections::HashMap;

    fn setup_test_snake(id: u128, initial_position: Vector2<f32>) -> (
        Snake,
        HashMap<RigidBodyHandle, RigidBody>,
        HashMap<RigidBodyHandle, Collider>,
        Vec<Option<RevoluteJoint>>
    ) {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        let mut impulse_joint_set = ImpulseJointSet::new();
        let mut snake = Snake::new(0.1, 5, 0.2);
        
        // Store components for later use
        let mut bodies = HashMap::new();
        let mut colliders = HashMap::new();
        let mut joints = Vec::new();

        // Create segments
        let mut parent_handle: Option<RigidBodyHandle> = None;
        let mut rng = rand::thread_rng();
        let initial_angle: f32 = rng.gen_range(-0.05..0.05);

        for i in 0..snake.segment_count {
            let segment_x = initial_position.x + (i as f32) * snake.segment_spacing * initial_angle.cos();
            let segment_y = initial_position.y + (i as f32) * snake.segment_spacing * initial_angle.sin();
            let orientation = initial_angle;

            // Create RigidBody
            let rb = RigidBodyBuilder::dynamic()
                .translation(vector![segment_x, segment_y])
                .rotation(orientation)
                .linear_damping(20.0)
                .angular_damping(10.0)
                .build();
            let segment_handle = rigid_body_set.insert(rb);
            bodies.insert(segment_handle, rigid_body_set.get(segment_handle).unwrap().clone());
            snake.segment_handles.push(segment_handle);

            // Create Collider
            let collider = ColliderBuilder::ball(snake.segment_radius)
                .restitution(0.0)
                .density(5.0)
                .friction(0.1)
                .user_data(id)
                .build();
            let collider_handle = collider_set.insert_with_parent(collider.clone(), segment_handle, &mut rigid_body_set);
            colliders.insert(segment_handle, collider);

            // Create joint
            if let Some(prev_handle) = parent_handle {
                let joint = RevoluteJointBuilder::new()
                    .local_anchor1(Point2::new(snake.segment_spacing / 2.0, 0.0))
                    .local_anchor2(Point2::new(-snake.segment_spacing / 2.0, 0.0))
                    .motor_velocity(0.0, 0.0)
                    .motor_max_force(0.5)
                    .motor_model(MotorModel::ForceBased)
                    .limits([-0.05, 0.05])
                    .build();
                let joint_handle = impulse_joint_set.insert(prev_handle, segment_handle, joint.clone(), true);
                snake.joint_handles.push(joint_handle);
                joints.push(Some(joint));
            } else {
                joints.push(None);
            }

            parent_handle = Some(segment_handle);
        }

        (snake, bodies, colliders, joints)
    }

    #[test]
    fn test_snake_movement_stability() {
        // Create physics pipeline and other required components
        let gravity = vector![0.0, 0.0];
        let mut physics_pipeline = PhysicsPipeline::new();
        let mut island_manager = IslandManager::new();
        let mut broad_phase = BroadPhaseMultiSap::new();
        let mut narrow_phase = NarrowPhase::new();
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        let mut impulse_joint_set = ImpulseJointSet::new();
        let mut multibody_joint_set = MultibodyJointSet::new();
        let mut ccd_solver = CCDSolver::new();
        let mut query_pipeline = QueryPipeline::new();

        // Create a single snake in the center
        let (mut snake, bodies, colliders, joints) = setup_test_snake(1, Vector2::new(0.0, 0.0));
        
        // Add snake bodies to the physics world
        for (old_handle, body) in bodies {
            let new_handle = rigid_body_set.insert(body);
            // Update the handle in the snake to point to the new body
            if let Some(pos) = snake.segment_handles.iter().position(|&h| h == old_handle) {
                snake.segment_handles[pos] = new_handle;
            }
        }

        // Add colliders to the physics world
        for (body_handle, collider) in colliders {
            if let Some(new_body_handle) = snake.segment_handles.iter().find(|&&h| h == body_handle) {
                collider_set.insert_with_parent(collider, *new_body_handle, &mut rigid_body_set);
            }
        }

        // Add joints to the physics world
        for (i, joint) in joints.iter().enumerate() {
            if let Some(joint) = joint {
                if i + 1 < snake.segment_handles.len() {
                    let parent_handle = snake.segment_handles[i];
                    let child_handle = snake.segment_handles[i + 1];
                    let new_joint = impulse_joint_set.insert(parent_handle, child_handle, joint.clone(), true);
                    snake.joint_handles[i] = new_joint;
                }
            }
        }
        
        // Create world context
        let world_context = WorldContext {
            world_height: 10.0,
            pixels_per_meter: 100.0,
        };

        // Track positions and velocities
        let mut positions: Vec<Vec<Vector2<f32>>> = Vec::new();
        let mut velocities: Vec<Vec<Vector2<f32>>> = Vec::new();
        let mut max_position_change: f32 = 0.0;
        let mut max_velocity_change: f32 = 0.0;
        let mut problematic_frames: Vec<usize> = Vec::new();
        let mut last_safe_frame: usize = 0;

        // Run simulation for 1000 steps
        for frame in 0..1000 {
            // Record current state
            let mut frame_positions = Vec::new();
            let mut frame_velocities = Vec::new();
            
            for handle in &snake.segment_handles {
                if let Some(body) = rigid_body_set.get(*handle) {
                    let pos = Vector2::new(body.translation().x, body.translation().y);
                    let vel = Vector2::new(body.linvel().x, body.linvel().y);
                    frame_positions.push(pos);
                    frame_velocities.push(vel);
                }
            }
            
            positions.push(frame_positions);
            velocities.push(frame_velocities);

            // Update snake
            snake.update_state_and_behavior(
                0.016, // 60 FPS
                1,
                &mut rigid_body_set,
                &mut impulse_joint_set,
                &collider_set,
                &query_pipeline,
                &Vec::new(),
                &world_context,
            );

            // Step the physics simulation
            physics_pipeline.step(
                &gravity,
                &IntegrationParameters::default(),
                &mut island_manager,
                &mut broad_phase,
                &mut narrow_phase,
                &mut rigid_body_set,
                &mut collider_set,
                &mut impulse_joint_set,
                &mut multibody_joint_set,
                &mut ccd_solver,
                Some(&mut query_pipeline),
                &(),
                &(),
            );

            // Check for sudden changes if we have previous frame data
            if frame > 0 {
                let prev_positions = &positions[frame - 1];
                let prev_velocities = &velocities[frame - 1];
                let curr_positions = &positions[frame];
                let curr_velocities = &velocities[frame];

                let mut frame_has_problem = false;

                // Check each segment
                for i in 0..curr_positions.len() {
                    // Calculate position change
                    let pos_change = (curr_positions[i] - prev_positions[i]).norm();
                    max_position_change = max_position_change.max(pos_change);

                    // Calculate velocity change
                    let vel_change = (curr_velocities[i] - prev_velocities[i]).norm();
                    max_velocity_change = max_velocity_change.max(vel_change);

                    // If change is too large, record the frame
                    if pos_change > 0.5 || vel_change > 5.0 {
                        frame_has_problem = true;
                        problematic_frames.push(frame);
                        println!("\nFrame {}: Segment {} had large change", frame, i);
                        println!("  Position change: {:.3} units", pos_change);
                        println!("  Velocity change: {:.3} units", vel_change);
                        println!("  Previous position: {:?}", prev_positions[i]);
                        println!("  Current position: {:?}", curr_positions[i]);
                        println!("  Previous velocity: {:?}", prev_velocities[i]);
                        println!("  Current velocity: {:?}", curr_velocities[i]);
                        
                        // Print joint states
                        if i < snake.joint_handles.len() {
                            if let Some(joint) = impulse_joint_set.get(snake.joint_handles[i]) {
                                println!("  Joint {} motor velocity: {:.3}", i, 
                                    joint.data.motor(JointAxis::AngX).unwrap().target_vel);
                            }
                        }

                        // Print snake state
                        println!("  Snake state: {:?}", snake.current_state);
                        println!("  Energy: {:.1}/{:.1}", 
                            snake.attributes.energy, 
                            snake.attributes.max_energy);
                    }
                }

                if !frame_has_problem {
                    last_safe_frame = frame;
                }
            }

            // Check if snake is still within bounds
            for (i, pos) in positions[frame].iter().enumerate() {
                if pos.x.abs() >= world_context.world_height/2.0 || 
                   pos.y.abs() >= world_context.world_height/2.0 {
                    println!("\nOUT OF BOUNDS at frame {}: Segment {}", frame, i);
                    println!("  Position: {:?}", pos);
                    println!("  Last safe frame: {}", last_safe_frame);
                    println!("  Frames since last safe: {}", frame - last_safe_frame);
                    panic!("Snake went out of bounds");
                }
            }
        }

        // Print summary
        println!("\nMovement Analysis Summary:");
        println!("Maximum position change per frame: {:.3}", max_position_change);
        println!("Maximum velocity change per frame: {:.3}", max_velocity_change);
        println!("Number of problematic frames: {}", problematic_frames.len());
        
        if !problematic_frames.is_empty() {
            println!("\nProblematic frames: {:?}", problematic_frames);
            
            // Analyze patterns in problematic frames
            let mut gaps = Vec::new();
            for i in 1..problematic_frames.len() {
                gaps.push(problematic_frames[i] - problematic_frames[i-1]);
            }
            if !gaps.is_empty() {
                println!("Average gap between problems: {:.1} frames", 
                    gaps.iter().sum::<usize>() as f32 / gaps.len() as f32);
            }
        }

        // Assert that changes weren't too drastic
        assert!(max_position_change < 1.0, "Position changes too large: {:.3}", max_position_change);
        assert!(max_velocity_change < 10.0, "Velocity changes too large: {:.3}", max_velocity_change);
    }
} 