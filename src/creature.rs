use eframe::egui;
use rapier2d::prelude::*;
use std::any::Any;

pub trait Creature {
    fn update_state(&mut self, ctx: &egui::Context);
    fn draw(&self, painter: &egui::Painter);
    fn get_segments(&self) -> &[Segment];
    fn get_segments_mut(&mut self) -> &mut [Segment];
    fn get_target_segments(&self) -> usize;
    fn set_target_segments(&mut self, count: usize);
    fn get_show_properties(&self) -> bool;
    fn set_show_properties(&mut self, show: bool);
    fn get_show_skin(&self) -> bool;
    fn set_show_skin(&mut self, show: bool);
    fn get_type_name(&self) -> &'static str;

    // New physics-related methods
    fn setup_physics(&mut self);
    fn update_physics(&mut self, dt: f32);
    fn get_rigid_body_handles(&self) -> &[RigidBodyHandle];
    fn get_joint_handles(&self) -> &[ImpulseJointHandle];
    fn as_any(&self) -> &dyn Any;

    fn show_properties(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Type: {}", self.get_type_name()));
        ui.separator();
        
        let mut target_segments = self.get_target_segments();
        if ui.add(egui::Slider::new(&mut target_segments, 1..=20).text("Target Segments")).changed() {
            self.set_target_segments(target_segments);
        }
        
        let mut show_skin = self.get_show_skin();
        if ui.checkbox(&mut show_skin, "Show Skin").changed() {
            self.set_show_skin(show_skin);
        }
    }
}

#[derive(Clone)]
pub struct Segment {
    pub pos: egui::Pos2,
    pub radius: f32,
    pub color: egui::Color32,
    pub left_point: egui::Pos2,
    pub right_point: egui::Pos2,
}

impl Segment {
    pub fn new(pos: egui::Pos2, radius: f32, color: egui::Color32) -> Self {
        Self {
            pos,
            radius,
            color,
            left_point: pos,
            right_point: pos,
        }
    }

    pub fn update_side_points(&mut self, next_pos: Option<egui::Pos2>, prev_pos: Option<egui::Pos2>) {
        let direction = if let Some(next) = next_pos {
            (next - self.pos).normalized()
        } else if let Some(prev) = prev_pos {
            // For the last segment, use the same direction as the previous segment
            (self.pos - prev).normalized()
        } else {
            egui::Vec2::new(1.0, 0.0)  // Default direction if no segments
        };

        // Calculate perpendicular vector (90 degrees rotation)
        let perpendicular = egui::Vec2::new(direction.y, -direction.x);

        // Update side points
        self.left_point = self.pos + perpendicular * self.radius;
        self.right_point = self.pos - perpendicular * self.radius;
    }
}

// Common physics world structure that can be shared between creatures
pub struct PhysicsWorld {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub physics_pipeline: PhysicsPipeline,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
    pub gravity: Vector<f32>,
    pub scale: f32,  // Pixels per meter
}

impl Clone for PhysicsWorld {
    fn clone(&self) -> Self {
        Self {
            rigid_body_set: self.rigid_body_set.clone(),
            collider_set: self.collider_set.clone(),
            joint_set: self.joint_set.clone(),
            multibody_joint_set: self.multibody_joint_set.clone(),
            island_manager: self.island_manager.clone(),
            broad_phase: self.broad_phase.clone(),
            narrow_phase: self.narrow_phase.clone(),
            physics_pipeline: PhysicsPipeline::new(), // Create new instance since it can't be cloned
            ccd_solver: self.ccd_solver.clone(),
            query_pipeline: self.query_pipeline.clone(),
            gravity: self.gravity,
            scale: self.scale,
        }
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            physics_pipeline: PhysicsPipeline::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            gravity: vector![0.0, 9.81],  // Standard gravity in m/s²
            scale: 50.0,  // 50 pixels = 1 meter
        }
    }
}

impl PhysicsWorld {
    pub fn step(&mut self, dt: f32) {
        let gravity = self.gravity * dt;
        let integration_parameters = IntegrationParameters::default();

        self.physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    pub fn create_segment_rigid_body(&mut self, pos: egui::Pos2, radius: f32) -> RigidBodyHandle {
        // Convert from pixels to meters
        let pos_meters = vector![pos.x / self.scale, pos.y / self.scale];
        let radius_meters = radius / self.scale;
        
        // Calculate mass in kg (using density of water as reference)
        let volume = std::f32::consts::PI * radius_meters * radius_meters;
        let mass = volume * 1000.0; // 1000 kg/m³ (water density)
        
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(pos_meters)
            .linear_damping(0.1)
            .angular_damping(0.1)
            .additional_mass(mass)
            .build();
        
        let handle = self.rigid_body_set.insert(rigid_body);

        // Create collider for the segment
        let collider = ColliderBuilder::ball(radius_meters)
            .restitution(0.1)
            .friction(0.3)
            .density(1000.0)  // 1000 kg/m³
            .build();
        
        self.collider_set.insert_with_parent(
            collider,
            handle,
            &mut self.rigid_body_set,
        );

        handle
    }

    pub fn create_segment_joint(&mut self, parent: RigidBodyHandle, child: RigidBodyHandle) -> ImpulseJointHandle {
        // Convert anchor points from pixels to meters
        let anchor1 = point![15.0 / self.scale, 0.0];
        let anchor2 = point![-15.0 / self.scale, 0.0];
        
        let joint = RevoluteJointBuilder::new()
            .local_anchor1(anchor1)
            .local_anchor2(anchor2)
            .limits([-1.0, 1.0])
            .build();
        
        self.joint_set.insert(
            parent,
            child,
            joint,
            true,
        )
    }
} 