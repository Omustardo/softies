use eframe::egui;
use rapier2d::prelude::*;
use nalgebra::Vector2;

use crate::creatures::snake::Snake; // Keep for initialization
use crate::creature::{Creature, CreatureState}; // Re-import CreatureState

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

        // --- Create Snake ---
        let segment_radius = 5.0 / PIXELS_PER_METER;
        let segment_spacing = 15.0 / PIXELS_PER_METER;
        let mut snake = Snake::new(
            segment_radius,
            10, // Number of segments
            segment_spacing,
        );

        // Spawn the snake (adjust initial y position)
        let snake_id = creatures.len() as u128; // ID will be 0
        snake.spawn_rapier(
            &mut rigid_body_set,
            &mut collider_set,
            &mut impulse_joint_set,
            Vector2::new(0.0, hh / 2.0), // Start in the upper half of the aquarium
            snake_id, // Pass the ID
        );
        creatures.push(Box::new(snake));


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
            creatures, // Store the vec
            view_center: Vector2::zeros(),
            zoom: 1.0,
            hovered_creature_id: None, // Initialize hover state
        }
    }
}

impl eframe::App for SoftiesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set dark theme explicitly
        ctx.set_visuals(egui::Visuals::dark());

        // Get delta time
        let dt = ctx.input(|i| i.stable_dt);

        // --- Creature Updates --- 
        // Determine if the snake is resting (e.g., head velocity is low)
        // Re-enable resting check
        let resting_velocity_threshold = 0.1; // Meters per second
        // Find the head velocity of the *first* creature for now (simplification)
        // TODO: Check resting state for each creature individually
        let is_snake_resting = 
            if let Some(creature) = self.creatures.first() { // Check if creatures exist
                if let Some(head_handle) = creature.get_rigid_body_handles().first() {
                    if let Some(head_body) = self.rigid_body_set.get(*head_handle) {
                        head_body.linvel().norm() < resting_velocity_threshold
                    } else { false } // Body not found
                } else { false } // No handles
            } else { false }; // No creatures
        // Remove the forced false value
        // let is_snake_resting = false; // Force not resting for now 

        // Update passive stats (hunger, energy recovery)
        for creature in &mut self.creatures {
            // Pass the actual resting state
            // TODO: Pass individual resting state for each creature later
            creature.attributes_mut().update_passive_stats(dt, is_snake_resting);
        }

        // Decide state and apply behavior (replaces simple actuation)
        for creature in &mut self.creatures {
            creature.update_state_and_behavior(
                dt, 
                &mut self.rigid_body_set, 
                &mut self.impulse_joint_set,
                &self.collider_set, // Pass collider set
            );
        }

        // --- Apply Custom Physics Forces --- 
        // This loop is now simpler. It calls the method on the Creature trait.
        // If a creature doesn't override the default empty method, nothing happens.
        // If it does (like Snake), its custom logic is executed.
        for creature in &self.creatures { // Need immutable borrow to call &self method
            creature.apply_custom_forces(&mut self.rigid_body_set);
        }

        // --- Physics Step --- 
        self.physics_pipeline.step(
            &Vector2::new(0.0, -1.0), // Reduced gravity for water environment
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None, // No query pipeline yet
            &self.physics_hooks,
            &self.event_handler,
        );

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
                );
            }
        });

        // Request redraw for animation
        ctx.request_repaint();
    }
}