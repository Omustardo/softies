use eframe::egui;
use rapier2d::prelude::*;
use nalgebra::{Point2, Vector2};

// Uncomment snake import and bring Creature trait into scope
use crate::creatures::snake::Snake;
use crate::creature::Creature;

const AQUARIUM_WIDTH: f32 = 1000.0;
const AQUARIUM_HEIGHT: f32 = 800.0;

pub struct SoftiesApp {
    // Rapier physics state
    gravity: Vector2<f32>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase, // Use concrete type
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: (),

    // Simulation state
    snakes: Vec<Snake>,

    // View state
    view_offset: egui::Vec2,
    zoom: f32,
}

impl Default for SoftiesApp {
    fn default() -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        let mut impulse_joint_set = ImpulseJointSet::new();

        // --- Initialize physics world (walls, etc.) ---
        let wall_thickness = 20.0;
        let half_width = AQUARIUM_WIDTH / 2.0;
        let half_height = AQUARIUM_HEIGHT / 2.0;
        let wall_friction = 0.7;

        // Left wall
        let left_wall_rb = RigidBodyBuilder::fixed().translation(vector![-half_width, 0.0]).build();
        let left_wall_handle = rigid_body_set.insert(left_wall_rb);
        let left_wall_collider = ColliderBuilder::cuboid(wall_thickness / 2.0, half_height)
                                    .friction(wall_friction)
                                    .build();
        collider_set.insert_with_parent(left_wall_collider, left_wall_handle, &mut rigid_body_set);

        // Right wall
        let right_wall_rb = RigidBodyBuilder::fixed().translation(vector![half_width, 0.0]).build();
        let right_wall_handle = rigid_body_set.insert(right_wall_rb);
        let right_wall_collider = ColliderBuilder::cuboid(wall_thickness / 2.0, half_height)
                                     .friction(wall_friction)
                                     .build();
        collider_set.insert_with_parent(right_wall_collider, right_wall_handle, &mut rigid_body_set);

        // Top wall
        let top_wall_rb = RigidBodyBuilder::fixed().translation(vector![0.0, half_height]).build();
        let top_wall_handle = rigid_body_set.insert(top_wall_rb);
        let top_wall_collider = ColliderBuilder::cuboid(half_width, wall_thickness / 2.0)
                                   .friction(wall_friction)
                                   .build();
        collider_set.insert_with_parent(top_wall_collider, top_wall_handle, &mut rigid_body_set);

        // Bottom wall
        let bottom_wall_rb = RigidBodyBuilder::fixed().translation(vector![0.0, -half_height]).build();
        let bottom_wall_handle = rigid_body_set.insert(bottom_wall_rb);
        let bottom_wall_collider = ColliderBuilder::cuboid(half_width, wall_thickness / 2.0)
                                      .friction(wall_friction)
                                      .build();
        collider_set.insert_with_parent(bottom_wall_collider, bottom_wall_handle, &mut rigid_body_set);

        // --- Initialize snake ---
        let mut snakes = Vec::new();
        let mut initial_snake = Snake::new(10.0, 10, 15.0);
        initial_snake.spawn_rapier(
            &mut rigid_body_set,
            &mut collider_set,
            &mut impulse_joint_set,
            vector![0.0, 100.0], // Start near top-center
        );
        snakes.push(initial_snake);

        Self {
            gravity: vector![0.0, -98.1],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(), // Use concrete type
            narrow_phase: NarrowPhase::new(),
            rigid_body_set,
            collider_set,
            impulse_joint_set,
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
            snakes,
            view_offset: egui::Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl eframe::App for SoftiesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Physics Step ---
        self.physics_pipeline.step(
            &self.gravity,
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

        // --- Drawing ---
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

            // --- Input Handling (Pan/Zoom - Basic) ---
            if response.dragged_by(egui::PointerButton::Primary) {
                self.view_offset += response.drag_delta();
            }
            let scroll_delta = ctx.input(|i| i.raw_scroll_delta); // Use raw_scroll_delta
            if scroll_delta.y != 0.0 {
                let mouse_pos = ctx.input(|i| i.pointer.hover_pos().unwrap_or(response.rect.center()));
                let zoom_delta = (scroll_delta.y / 100.0).exp();

                let world_pos_before = screen_to_world(mouse_pos, response.rect.center(), self.view_offset, self.zoom);
                self.zoom *= zoom_delta;
                self.zoom = self.zoom.clamp(0.1, 5.0);
                let world_pos_after = screen_to_world(mouse_pos, response.rect.center(), self.view_offset, self.zoom);

                // Convert nalgebra::Vector2 to egui::Vec2 for view_offset update
                let offset_diff_nalgebra = (world_pos_after - world_pos_before) * self.zoom;
                let offset_diff_egui = egui::vec2(offset_diff_nalgebra.x, offset_diff_nalgebra.y);
                self.view_offset += offset_diff_egui;
            }

            // --- World to Screen Transformation ---
            let world_to_screen = |world_pos: &Point2<f32>, rect_center: egui::Pos2, view_offset: egui::Vec2, zoom: f32| -> egui::Pos2 {
                let zoomed_pos = world_pos * zoom;
                let screen_pos = egui::pos2(zoomed_pos.x, -zoomed_pos.y);
                rect_center + screen_pos.to_vec2() + view_offset
            };

            let center = response.rect.center();

            // --- Draw Walls (Debug Example) ---
            for (_handle, collider) in self.collider_set.iter() {
                 if let Some(parent_handle) = collider.parent() {
                     if let Some(rb) = self.rigid_body_set.get(parent_handle) {
                         if rb.body_type() == RigidBodyType::Fixed {
                             if let Some(cuboid) = collider.shape().as_cuboid() {
                                  let half_extents = cuboid.half_extents * self.zoom;
                                  let pos = rb.translation();
                                  let screen_pos = world_to_screen(&Point2::new(pos.x, pos.y), center, self.view_offset, self.zoom);
                                  let rect = egui::Rect::from_center_size(
                                      screen_pos,
                                      egui::vec2(half_extents.x * 2.0, half_extents.y * 2.0),
                                  );
                                  painter.rect_stroke(
                                      rect,
                                      egui::Rounding::ZERO,
                                      egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
                                  );
                             }
                         }
                     }
                 }
            }

            // --- Draw Snakes ---
            for snake in &self.snakes {
                let mut points: Vec<egui::Pos2> = Vec::new();
                // Use the Creature trait method now that it's in scope
                for segment_handle in snake.get_rigid_body_handles() {
                    if let Some(rb) = self.rigid_body_set.get(*segment_handle) {
                        let pos = rb.translation();
                        let radius = snake.segment_radius * self.zoom;
                        let screen_pos = world_to_screen(&Point2::new(pos.x, pos.y), center, self.view_offset, self.zoom);
                        points.push(screen_pos);

                        // Draw the individual segment circle
                        painter.circle_filled(
                            screen_pos,
                            radius.max(1.0), // Ensure radius is visible
                            egui::Color32::GREEN,
                        );
                    }
                }
                // Draw lines connecting segment centers for the "skeleton"
                if points.len() > 1 {
                    painter.add(
                        egui::Shape::line(
                            points,
                            egui::Stroke::new((2.0 * self.zoom).max(1.0), egui::Color32::LIGHT_GREEN), // Ensure stroke is visible
                        )
                    );
                }
                // TODO: Implement drawing for the "smooth outside" based on these points
            }

            ctx.request_repaint();
        });
    }
}

// Helper function for screen to world conversion (for zoom-to-cursor)
fn screen_to_world(screen_pos: egui::Pos2, rect_center: egui::Pos2, view_offset: egui::Vec2, zoom: f32) -> Vector2<f32> {
    if zoom == 0.0 { return Vector2::zeros(); }
    let center_offset = (screen_pos - rect_center - view_offset) / zoom;
    vector![center_offset.x, -center_offset.y]
} 