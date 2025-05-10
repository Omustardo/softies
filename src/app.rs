use eframe::egui;
use rapier2d::prelude::*;
use nalgebra::{Vector2, Rotation2}; // Added Rotation2
use rand::Rng; // Import random number generator

use crate::creatures::snake::Snake; // Keep for initialization
use crate::creatures::plankton::Plankton; // Import Plankton
use crate::creature::{Creature, CreatureInfo, WorldContext}; // Added CreatureInfo and WorldContext explicitly

// Constants for the simulation world
const PIXELS_PER_METER: f32 = 50.0;
const WORLD_WIDTH_METERS: f32 = 20.0; // e.g., 1000 pixels / 50 px/m = 20m
const WORLD_HEIGHT_METERS: f32 = 16.0; // e.g., 800 pixels / 50 px/m = 16m
const WALL_THICKNESS: f32 = 0.5; // Half a meter thick walls

// Unused for now, but keep for reference
// const TIMESTEP: f32 = 1.0 / 60.0; // Run physics at 60Hz

pub struct SoftiesApp {
    // Rapier physics world components
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhaseMultiSap,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline, // Added query pipeline
    physics_hooks: (), // No hooks for now
    event_handler: (), // No events for now

    // Creatures
    creatures: Vec<Box<dyn Creature>>, // Changed from single snake

    // View state (optional, for panning/zooming later)
    view_center: Vector2<f32>,
    zoom: f32,

    // UI State
    hovered_creature_id: Option<usize>,
}

impl Default for SoftiesApp {
    fn default() -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        let mut impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let query_pipeline = QueryPipeline::new(); // Initialize query pipeline

        // --- Create Walls ---
        let hw = WORLD_WIDTH_METERS / 2.0;
        let hh = WORLD_HEIGHT_METERS / 2.0;
        let wt = WALL_THICKNESS / 2.0;

        // Floor
        let floor_rb = RigidBodyBuilder::fixed().translation(vector![0.0, -hh - wt]).build();
        let floor_handle = rigid_body_set.insert(floor_rb);
        let floor_collider = ColliderBuilder::cuboid(hw + wt, wt).user_data(u128::MAX); // Assign high user_data to walls
        collider_set.insert_with_parent(floor_collider, floor_handle, &mut rigid_body_set);

        // Ceiling
        let ceiling_rb = RigidBodyBuilder::fixed().translation(vector![0.0, hh + wt]).build();
        let ceiling_handle = rigid_body_set.insert(ceiling_rb);
        let ceiling_collider = ColliderBuilder::cuboid(hw + wt, wt).user_data(u128::MAX);
        collider_set.insert_with_parent(ceiling_collider, ceiling_handle, &mut rigid_body_set);

        // Left Wall
        let left_wall_rb = RigidBodyBuilder::fixed().translation(vector![-hw - wt, 0.0]).build();
        let left_wall_handle = rigid_body_set.insert(left_wall_rb);
        let left_wall_collider = ColliderBuilder::cuboid(wt, hh + wt).user_data(u128::MAX);
        collider_set.insert_with_parent(left_wall_collider, left_wall_handle, &mut rigid_body_set);

        // Right Wall
        let right_wall_rb = RigidBodyBuilder::fixed().translation(vector![hw + wt, 0.0]).build();
        let right_wall_handle = rigid_body_set.insert(right_wall_rb);
        let right_wall_collider = ColliderBuilder::cuboid(wt, hh + wt).user_data(u128::MAX);
        collider_set.insert_with_parent(right_wall_collider, right_wall_handle, &mut rigid_body_set);


        // --- Create Creatures ---
        let mut creatures: Vec<Box<dyn Creature>> = Vec::new();
        let mut creature_id_counter: u128 = 0;
        let mut rng = rand::thread_rng(); // Initialize RNG

        // --- Create Multiple Snakes ---
        let num_snakes = 3;
        let segment_radius = 5.0 / PIXELS_PER_METER;
        let segment_spacing = 15.0 / PIXELS_PER_METER;
        let margin = 2.0; // Keep snakes away from walls

        for i in 0..num_snakes {
            let mut snake = Snake::new(
                segment_radius,
                10, // Number of segments
                segment_spacing,
            );

            // Adjust energy parameters for longer active periods
            snake.attributes_mut().max_energy = 150.0; // Increased from 100.0
            snake.attributes_mut().energy_recovery_rate = 8.0; // Increased from 5.0
            snake.attributes_mut().metabolic_rate = 0.5; // Reduced from 1.0
            snake.attributes_mut().energy = 150.0; // Start with full energy

            // Calculate different starting positions for each snake
            let initial_x = match i {
                0 => -hw / 2.0, // Left side
                1 => 0.0,       // Center
                2 => hw / 2.0,  // Right side
                _ => rng.gen_range((-hw + margin)..(hw - margin)), // Random for any additional snakes
            };
            let initial_y = match i {
                0 => hh / 3.0,  // Upper third
                1 => 0.0,       // Middle
                2 => -hh / 3.0, // Lower third
                _ => rng.gen_range((-hh + margin)..(hh - margin)), // Random for any additional snakes
            };

            snake.spawn_rapier(
                &mut rigid_body_set,
                &mut collider_set,
                &mut impulse_joint_set,
                Vector2::new(initial_x, initial_y),
                creature_id_counter,
            );
            creatures.push(Box::new(snake));
            creature_id_counter += 1;
        }

        // --- Create Plankton ---
        let num_plankton = 20;
        let plankton_radius = 4.0 / PIXELS_PER_METER; // Made smaller
        for _ in 0..num_plankton {
            let mut plankton = Plankton::new(plankton_radius);
            // Random position
            let margin = 1.0;
            let initial_x = rng.gen_range((-hw + margin)..(hw - margin));
            let initial_y = rng.gen_range((-hh + margin)..(hh - margin));
            
            plankton.spawn_rapier(
                &mut rigid_body_set,
                &mut collider_set,
                &mut impulse_joint_set, // Pass joint set
                Vector2::new(initial_x, initial_y),
                creature_id_counter,
            );
            creatures.push(Box::new(plankton));
            creature_id_counter += 1;
        }


        Self {
            rigid_body_set,
            collider_set,
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseMultiSap::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver: CCDSolver::new(),
            query_pipeline, // Store query pipeline
            physics_hooks: (),
            event_handler: (),
            creatures, // Store the vec containing snake and plankton
            view_center: Vector2::zeros(),
            zoom: 1.0,
            hovered_creature_id: None, // Initialize hover state
        }
    }
}

impl SoftiesApp {
    // Add the new tick_simulation method here, before eframe::App impl
    pub fn tick_simulation(&mut self, dt: f32, _ctx: &egui::Context) {
        // --- Creature Updates --- 
        for creature in &mut self.creatures {
            let is_this_creature_resting = creature.current_state() == crate::creature::CreatureState::Resting;
            creature.attributes_mut().update_passive_stats(dt, is_this_creature_resting);
        }

        // --- Prepare CreatureInfo vector --- 
        let mut all_creatures_info: Vec<CreatureInfo> = Vec::with_capacity(self.creatures.len());
        for (_index, creature) in self.creatures.iter().enumerate() {
            let creature_id = creature.id(); 
            let type_name = creature.type_name();
            let radius = creature.drawing_radius();
            let primary_body_handle = creature.get_rigid_body_handles().first().cloned().unwrap_or_else(RigidBodyHandle::invalid);
            
            let (position, velocity) = if primary_body_handle != RigidBodyHandle::invalid() {
                if let Some(body) = self.rigid_body_set.get(primary_body_handle) {
                    (*body.translation(), *body.linvel())
                } else {
                    (Vector2::zeros(), Vector2::zeros())
                }
            } else {
                (Vector2::zeros(), Vector2::zeros())
            };

            all_creatures_info.push(CreatureInfo {
                id: creature_id,
                creature_type_name: type_name,
                primary_body_handle,
                position,
                velocity,
                radius,
            });
        }

        // Decide state and apply behavior
        for creature in &mut self.creatures {
            let world_context = WorldContext { 
                world_height: WORLD_HEIGHT_METERS,
                pixels_per_meter: PIXELS_PER_METER, 
            };
            
            let own_id = creature.id();

            creature.update_state_and_behavior(
                dt, 
                own_id, 
                &mut self.rigid_body_set, 
                &mut self.impulse_joint_set,
                &self.collider_set, 
                &self.query_pipeline,
                &all_creatures_info, 
                &world_context,
            );
        }

        // --- Apply Custom Physics Forces --- 
        let world_context_for_forces = crate::creature::WorldContext {
            world_height: WORLD_HEIGHT_METERS,
            pixels_per_meter: PIXELS_PER_METER,
        };
        for creature in &self.creatures { 
            creature.apply_custom_forces(&mut self.rigid_body_set, &world_context_for_forces);
        }

        // --- Physics Step --- 
        self.physics_pipeline.step(
            &Vector2::new(0.0, -1.0), 
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None, 
            &self.physics_hooks,
            &self.event_handler,
        );

        // --- Failsafe: Check for Escaped Creatures ---
        let world_half_width = WORLD_WIDTH_METERS / 2.0;
        let world_half_height = WORLD_HEIGHT_METERS / 2.0;
        let bounds_padding = 1.0;

        for (id, creature) in self.creatures.iter().enumerate() { 
            let mut is_out_of_bounds = false;
            for &body_handle in creature.get_rigid_body_handles() {
                if let Some(body) = self.rigid_body_set.get(body_handle) {
                    let pos = body.translation();
                    if pos.x.abs() > world_half_width + bounds_padding || 
                       pos.y.abs() > world_half_height + bounds_padding {
                        is_out_of_bounds = true;
                        break; 
                    }
                }
            }

            if is_out_of_bounds {
                eprintln!(
                    "WARN: Creature ID {} (Type: {}) escaped bounds and was reset!",
                    id, 
                    creature.type_name()
                );
                for &body_handle in creature.get_rigid_body_handles() {
                    if let Some(body) = self.rigid_body_set.get_mut(body_handle) {
                        body.set_translation(Vector2::zeros(), true);
                        body.set_linvel(Vector2::zeros(), true);
                        body.set_angvel(0.0, true);
                    }
                }
            }
        }

        // --- UI Panel and Drawing --- 
        // These parts will remain in the eframe::App::update method
        // as they interact directly with egui panels and painters.

        // Request redraw for animation (can also be in tick_simulation if preferred)
        // For now, let's keep it here, but it will be called by the main update loop.
        // ctx.request_repaint(); 
        // Actually, this should probably be in the main update function, 
        // as tick_simulation is just about the logic.
    }
}

impl eframe::App for SoftiesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set dark theme explicitly
        ctx.set_visuals(egui::Visuals::dark());

        // Get delta time
        let dt = ctx.input(|i| i.stable_dt);

        // Run the core simulation logic
        self.tick_simulation(dt, ctx);

        // --- UI Panel --- 
        egui::SidePanel::left("creature_list_panel")
            .resizable(true)
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.heading("Creatures");
                ui.separator();

                let mut currently_hovered: Option<usize> = None;
                for (id, creature) in self.creatures.iter().enumerate() {
                    let label_text = format!(
                        "ID: {}\nType: {}\nState: {:?}", 
                        id, 
                        creature.type_name(),
                        creature.current_state()
                    );
                    // Use selectable label for hover detection
                    let response = ui.selectable_label(false, label_text);
                    if response.hovered() {
                        currently_hovered = Some(id);
                    }
                    ui.separator();
                }
                // Update the app state *after* checking all labels
                self.hovered_creature_id = currently_hovered;
            });

        // --- Drawing --- 
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            let available_rect = ui.available_rect_before_wrap();

            // Simple world-to-screen transformation
            let world_to_screen = |world_pos: Vector2<f32>| -> egui::Pos2 {
                // Note: Using nalgebra's Point2 for clarity in transformations
                let world_pt = nalgebra::Point2::new(world_pos.x, world_pos.y);
                
                // 1. Apply view center offset (physics coords)
                let centered_pt = world_pt - self.view_center;
                // 2. Apply zoom 
                let zoomed_pt = centered_pt * self.zoom;
                // 3. Scale to screen pixels
                let pixel_pt = zoomed_pt * PIXELS_PER_METER;
                // 4. Convert to egui coordinates (origin top-left, Y down)
                //    relative to the center of the available rect
                let screen_center = available_rect.center();
                egui::pos2(screen_center.x + pixel_pt.x, screen_center.y - pixel_pt.y) // Invert Y here
            };

            // --- Draw Walls ---
            for (_collider_handle, collider) in self.collider_set.iter() { // Renamed handle to _collider_handle as it's not used directly here for fetching body
                if collider.user_data == u128::MAX { // Corrected: user_data is a field
                    if let Some(rigid_body_handle) = collider.parent() { // Get the parent RigidBodyHandle
                        if let Some(body) = self.rigid_body_set.get(rigid_body_handle) { // Use the RigidBodyHandle
                            let position = body.translation();
                            let rotation_angle = body.rotation().angle();

                            if let Some(cuboid) = collider.shape().as_cuboid() {
                                let half_extents = cuboid.half_extents;
                                // Helper to create rotated points
                                let create_rotated_point = |x_offset, y_offset| -> Vector2<f32> {
                                    Rotation2::new(rotation_angle) * Vector2::new(x_offset, y_offset)
                                };

                                let screen_points = [
                                    world_to_screen(*position + create_rotated_point(-half_extents.x, -half_extents.y)),
                                    world_to_screen(*position + create_rotated_point(half_extents.x, -half_extents.y)),
                                    world_to_screen(*position + create_rotated_point(half_extents.x, half_extents.y)),
                                    world_to_screen(*position + create_rotated_point(-half_extents.x, half_extents.y)),
                                ];

                                painter.add(egui::Shape::closed_line(
                                    screen_points.to_vec(),
                                    egui::Stroke::new(2.0, egui::Color32::GRAY)
                                ));
                            }
                        }
                    }
                }
            }

            // Draw the creatures
            for (id, creature) in self.creatures.iter().enumerate() {
                let is_hovered = self.hovered_creature_id == Some(id);
                
                // Call the creature's draw method
                creature.draw(
                    painter,
                    &self.rigid_body_set,
                    &world_to_screen, // Pass the closure
                    self.zoom,
                    is_hovered,
                    PIXELS_PER_METER, // Pass the constant
                );
            }
        });

        // Request redraw for animation
        ctx.request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Imports SoftiesApp, PIXELS_PER_METER, WORLD_HEIGHT_METERS etc.
    use crate::creature::CreatureState;
    use egui;   // For egui::Context and other egui types used in DummyFrame

    #[test]
    fn plankton_eventually_rests() {
        let mut app = SoftiesApp::default();
        let mock_ctx = egui::Context::default();

        // Set initial energy of plankton to be low, so they become tired faster.
        // Tired threshold is typically 20% of max_energy.
        // Plankton max_energy is 20.0, so tired at <= 4.0.
        // Start them at 22% (4.4 energy) so they are not immediately tired.
        for creature_box in app.creatures.iter_mut() {
            if creature_box.type_name() == "Plankton" {
                let max_energy = creature_box.attributes().max_energy;
                creature_box.attributes_mut().energy = max_energy * 0.22;
            }
        }

        let mut resting_observed = false;
        let iterations = 2000; // Increased from 1000
        let fixed_dt = 1.0 / 60.0; // Simulate at 60 FPS for the test

        for i in 0..iterations {
            app.tick_simulation(fixed_dt, &mock_ctx); // Call the new method

            for creature in &app.creatures {
                if creature.type_name() == "Plankton" {
                    if creature.current_state() == CreatureState::Resting {
                        println!("Plankton entered resting state at iteration {}", i);
                        resting_observed = true;
                        break;
                    }
                }
            }
            if resting_observed {
                break;
            }
        }
        assert!(resting_observed, "Plankton did not enter Resting state after {} iterations", iterations);
    }
}