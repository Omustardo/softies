use eframe::egui;
use rapier2d::prelude::*;
use nalgebra::Vector2;

use crate::creatures::Snake; // Import the Snake creature
use crate::creature::Creature; // Import the trait

// Constants for the simulation world
const PIXELS_PER_METER: f32 = 50.0; // Let's make things a bit bigger
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
    physics_hooks: (), // No hooks for now
    event_handler: (), // No events for now

    // Creatures
    snake: Snake, // Store the snake instance

    // View state (optional, for panning/zooming later)
    view_center: Vector2<f32>,
    zoom: f32,
}

impl Default for SoftiesApp {
    fn default() -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        let mut impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();

        // --- Create Walls ---
        let hw = WORLD_WIDTH_METERS / 2.0;
        let hh = WORLD_HEIGHT_METERS / 2.0;
        let wt = WALL_THICKNESS / 2.0;

        // Floor
        let floor_rb = RigidBodyBuilder::fixed().translation(vector![0.0, -hh - wt]).build();
        let floor_handle = rigid_body_set.insert(floor_rb);
        let floor_collider = ColliderBuilder::cuboid(hw + wt, wt).build(); // Extend width slightly
        collider_set.insert_with_parent(floor_collider, floor_handle, &mut rigid_body_set);

        // Ceiling
        let ceiling_rb = RigidBodyBuilder::fixed().translation(vector![0.0, hh + wt]).build();
        let ceiling_handle = rigid_body_set.insert(ceiling_rb);
        let ceiling_collider = ColliderBuilder::cuboid(hw + wt, wt).build(); // Extend width slightly
        collider_set.insert_with_parent(ceiling_collider, ceiling_handle, &mut rigid_body_set);

        // Left Wall
        let left_wall_rb = RigidBodyBuilder::fixed().translation(vector![-hw - wt, 0.0]).build();
        let left_wall_handle = rigid_body_set.insert(left_wall_rb);
        let left_wall_collider = ColliderBuilder::cuboid(wt, hh + wt).build(); // Extend height slightly
        collider_set.insert_with_parent(left_wall_collider, left_wall_handle, &mut rigid_body_set);

        // Right Wall
        let right_wall_rb = RigidBodyBuilder::fixed().translation(vector![hw + wt, 0.0]).build();
        let right_wall_handle = rigid_body_set.insert(right_wall_rb);
        let right_wall_collider = ColliderBuilder::cuboid(wt, hh + wt).build(); // Extend height slightly
        collider_set.insert_with_parent(right_wall_collider, right_wall_handle, &mut rigid_body_set);

        // --- Create Snake ---
        let segment_radius = 5.0 / PIXELS_PER_METER;
        let segment_spacing = 15.0 / PIXELS_PER_METER;
        let mut snake = Snake::new(
            segment_radius,
            10, // Number of segments
            segment_spacing,
        );

        // Spawn the snake (adjust initial y position)
        snake.spawn_rapier(
            &mut rigid_body_set,
            &mut collider_set,
            &mut impulse_joint_set,
            Vector2::new(0.0, hh / 2.0), // Start in the upper half of the aquarium
        );

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
            physics_hooks: (),
            event_handler: (),
            snake,
            view_center: Vector2::zeros(),
            zoom: 1.0,
        }
    }
}

impl eframe::App for SoftiesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set dark theme explicitly
        ctx.set_visuals(egui::Visuals::dark());

        // Step the physics simulation
        self.physics_pipeline.step(
            &Vector2::new(0.0, -9.81), // Use standard gravity now
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

            // --- Draw Walls (Optional Debug) ---
            for (_handle, collider) in self.collider_set.iter() {
                 if let Some(parent_handle) = collider.parent() {
                     if let Some(rb) = self.rigid_body_set.get(parent_handle) {
                         if rb.body_type() == RigidBodyType::Fixed {
                             if let Some(cuboid) = collider.shape().as_cuboid() {
                                  let half_extents = cuboid.half_extents;
                                  let pos = rb.translation();
                                  // Calculate screen coords for corners
                                  let top_left_world = Vector2::new(pos.x - half_extents.x, pos.y + half_extents.y);
                                  let bottom_right_world = Vector2::new(pos.x + half_extents.x, pos.y - half_extents.y);
                                  let top_left_screen = world_to_screen(top_left_world);
                                  let bottom_right_screen = world_to_screen(bottom_right_world);
                                  // Create egui::Rect (handle potential inversion)
                                  let wall_rect = egui::Rect::from_min_max(top_left_screen, bottom_right_screen);
                                  
                                  painter.rect_stroke(
                                      wall_rect,
                                      egui::Rounding::ZERO,
                                      egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
                                  );
                             }
                         }
                     }
                 }
            }
            
            // Draw the snake segments
            for handle in self.snake.get_rigid_body_handles() {
                if let Some(body) = self.rigid_body_set.get(*handle) {
                    let pos = body.translation();
                    let screen_pos = world_to_screen(Vector2::new(pos.x, pos.y));
                    let screen_radius = self.snake.segment_radius * PIXELS_PER_METER * self.zoom;

                    painter.circle_filled(
                        screen_pos,
                        screen_radius,
                        egui::Color32::GREEN, // Simple green color for now
                    );
                }
            }
        });

        // Request redraw for animation
        ctx.request_repaint();
    }
}