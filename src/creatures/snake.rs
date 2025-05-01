use eframe::egui;
use rapier2d::prelude::*;
use crate::{creature::{Creature, Segment, PhysicsWorld}, creature_ui::CreatureUI};
use std::any::Any;

pub struct Snake {
    segments: Vec<Segment>,
    target_segments: usize,
    show_properties: bool,
    show_skin: bool,
    time: f32,
    center: egui::Pos2,
    direction: egui::Vec2,
    speed: f32,
    ui: CreatureUI,
    
    // Physics components
    physics_world: PhysicsWorld,
    rigid_body_handles: Vec<RigidBodyHandle>,
    joint_handles: Vec<ImpulseJointHandle>,
}

impl Clone for Snake {
    fn clone(&self) -> Self {
        let mut new_snake = Self {
            segments: self.segments.clone(),
            target_segments: self.target_segments,
            show_properties: self.show_properties,
            show_skin: self.show_skin,
            time: self.time,
            center: self.center,
            direction: self.direction,
            speed: self.speed,
            ui: self.ui.clone(),
            physics_world: PhysicsWorld::default(),
            rigid_body_handles: Vec::new(),
            joint_handles: Vec::new(),
        };
        
        // Set up physics for the cloned snake
        new_snake.setup_physics();
        
        new_snake
    }
}

impl Default for Snake {
    fn default() -> Self {
        let mut segments = Vec::new();
        let start_pos = egui::Pos2::new(400.0, 300.0);
        let mut current_pos = start_pos;
        let direction = egui::Vec2::new(1.0, 0.0);  // Start moving right

        // Create initial segments
        for i in 0..5 {  // Reduced from 8 to 5 segments
            segments.push(Segment::new(
                current_pos,
                if i == 0 { 8.0 } else { 6.0 },
                if i == 0 {
                    egui::Color32::from_rgb(200, 100, 100)  // Red for head
                } else {
                    egui::Color32::from_rgb(100, 200, 100)  // Green for body
                },
            ));
            current_pos = current_pos + direction * 30.0;  // Increased spacing to match TestChain
        }

        let mut physics_world = PhysicsWorld::default();
        let mut rigid_body_handles = Vec::new();
        let mut joint_handles = Vec::new();

        // Create physics bodies for initial segments
        for (i, segment) in segments.iter().enumerate() {
            let rigid_body = RigidBodyBuilder::dynamic()
                .translation(vector![segment.pos.x, segment.pos.y])
                .linear_damping(0.5)
                .angular_damping(0.5)
                .lock_rotations()  // Prevent segments from rotating
                .dominance_group(if i == 0 { 1 } else { 0 })  // Head has higher dominance
                .build();
            
            let handle = physics_world.rigid_body_set.insert(rigid_body);
            rigid_body_handles.push(handle);

            // Create collider for the segment
            let collider = ColliderBuilder::ball(segment.radius)
                .restitution(0.2)
                .friction(0.7)
                .build();
            
            physics_world.collider_set.insert_with_parent(
                collider,
                handle,
                &mut physics_world.rigid_body_set,
            );
        }

        // Create joints between segments
        for i in 1..rigid_body_handles.len() {
            let joint = RevoluteJointBuilder::new()
                .local_anchor1(point![15.0, 0.0])
                .local_anchor2(point![-15.0, 0.0])
                .limits([-0.2, 0.2])  // Tighter rotation limits
                .build();
            
            let handle = physics_world.joint_set.insert(
                rigid_body_handles[i - 1],
                rigid_body_handles[i],
                joint,
                true,
            );
            joint_handles.push(handle);
        }

        Self {
            segments,
            target_segments: 8,
            show_properties: false,
            show_skin: true,
            time: 0.0,
            center: egui::Pos2::new(400.0, 300.0),
            direction,
            speed: 300.0,  // Increased speed
            ui: CreatureUI::new("snake"),
            physics_world,
            rigid_body_handles,
            joint_handles,
        }
    }
}

impl Creature for Snake {
    fn update_state(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.unstable_dt);
        if dt > 0.0 {
            self.time += dt;

            // Simple circular motion like TestChain
            let angle = self.time * 2.0;
            let direction = egui::Vec2::new(angle.cos(), angle.sin());
            self.direction = direction;

            // Apply force to the head to make it move
            if let Some(head_handle) = self.rigid_body_handles.first() {
                if let Some(head) = self.physics_world.rigid_body_set.get_mut(*head_handle) {
                    let force = vector![direction.x * 20.0, direction.y * 20.0];
                    head.add_force(force, true);
                    
                    // Set velocity directly for more control
                    let target_vel = vector![direction.x * 100.0, direction.y * 100.0];
                    head.set_linvel(target_vel, true);
                }
            }

            // Step physics simulation
            self.physics_world.step(dt);

            // Update segment positions from physics
            for (i, handle) in self.rigid_body_handles.iter().enumerate() {
                if let Some(rb) = self.physics_world.rigid_body_set.get(*handle) {
                    let pos = rb.translation();
                    self.segments[i].pos = egui::Pos2::new(pos.x, pos.y);
                    
                    // Update side points
                    let next_pos = if i < self.segments.len() - 1 {
                        Some(self.segments[i + 1].pos)
                    } else {
                        None
                    };
                    let prev_pos = if i > 0 {
                        Some(self.segments[i - 1].pos)
                    } else {
                        None
                    };
                    self.segments[i].update_side_points(next_pos, prev_pos);
                }
            }
        }

        // Adjust number of segments if needed
        while self.segments.len() < self.target_segments {
            let last_pos = self.segments.last().unwrap().pos;
            let direction = if self.segments.len() > 1 {
                (self.segments[self.segments.len()-2].pos - last_pos).normalized()
            } else {
                self.direction
            };
            let new_pos = last_pos + direction * 50.0;
            
            // Create new segment
            self.segments.push(Segment::new(
                new_pos,
                10.0,
                egui::Color32::from_rgb(100, 200, 100),
            ));

            // Create physics body for new segment
            let handle = self.physics_world.create_segment_rigid_body(new_pos, 10.0);
            self.rigid_body_handles.push(handle);

            // Create joint to previous segment
            if let Some(prev_handle) = self.rigid_body_handles.get(self.rigid_body_handles.len() - 2) {
                let joint = self.physics_world.create_segment_joint(*prev_handle, handle);
                self.joint_handles.push(joint);
            }
        }

        while self.segments.len() > self.target_segments {
            self.segments.pop();
            if let Some(handle) = self.rigid_body_handles.pop() {
                self.physics_world.rigid_body_set.remove(
                    handle,
                    &mut self.physics_world.island_manager,
                    &mut self.physics_world.collider_set,
                    &mut self.physics_world.joint_set,
                    &mut self.physics_world.multibody_joint_set,
                    true,
                );
            }
            if let Some(joint) = self.joint_handles.pop() {
                self.physics_world.joint_set.remove(joint, true);
            }
        }
    }

    fn draw(&self, painter: &egui::Painter) {
        // Pre-allocate vectors for better performance
        let mut shapes = Vec::with_capacity(self.segments.len() * 2);
        
        // Draw the skeleton first
        for segment in &self.segments {
            // Add main circle
            shapes.push(egui::Shape::circle_filled(
                segment.pos,
                segment.radius,
                segment.color,
            ));

            // Add side points
            shapes.push(egui::Shape::circle_filled(
                segment.left_point,
                3.0,
                egui::Color32::from_rgb(255, 255, 255),
            ));
            shapes.push(egui::Shape::circle_filled(
                segment.right_point,
                3.0,
                egui::Color32::from_rgb(255, 255, 255),
            ));
        }

        // Add connecting lines
        for i in 0..self.segments.len() - 1 {
            shapes.push(egui::Shape::line_segment(
                [self.segments[i].pos, self.segments[i + 1].pos],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 100)),
            ));
        }

        // Draw the skin if enabled
        if self.show_skin && self.segments.len() >= 2 {
            // Create fill polygons between adjacent segments
            for i in 0..self.segments.len() - 1 {
                let mut segment_points = Vec::with_capacity(4);
                segment_points.push(self.segments[i].left_point);
                segment_points.push(self.segments[i].right_point);
                segment_points.push(self.segments[i + 1].right_point);
                segment_points.push(self.segments[i + 1].left_point);
                
                shapes.push(egui::Shape::convex_polygon(
                    segment_points,
                    egui::Color32::from_rgba_premultiplied(100, 200, 100, 64),
                    egui::Stroke::NONE,
                ));
            }

            // Draw all side lines in a single shape
            let mut side_points = Vec::with_capacity(self.segments.len() * 2);
            // Add left side points from head to tail
            for segment in &self.segments {
                side_points.push(segment.left_point);
            }
            // Add right side points from tail to head
            for segment in self.segments.iter().rev() {
                side_points.push(segment.right_point);
            }
            shapes.push(egui::Shape::line(
                side_points,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 160, 80)),
            ));
        }

        // Draw all shapes in a single batch
        painter.extend(shapes);
    }

    fn get_segments(&self) -> &[Segment] {
        &self.segments
    }

    fn get_segments_mut(&mut self) -> &mut [Segment] {
        &mut self.segments
    }

    fn get_target_segments(&self) -> usize {
        self.target_segments
    }

    fn set_target_segments(&mut self, count: usize) {
        self.target_segments = count;
    }

    fn get_show_properties(&self) -> bool {
        self.show_properties
    }

    fn set_show_properties(&mut self, show: bool) {
        self.show_properties = show;
    }

    fn get_show_skin(&self) -> bool {
        self.show_skin
    }

    fn set_show_skin(&mut self, show: bool) {
        self.show_skin = show;
    }

    fn get_type_name(&self) -> &'static str {
        "Snake"
    }

    fn setup_physics(&mut self) {
        // Clear existing physics objects
        self.physics_world = PhysicsWorld::default();
        self.rigid_body_handles.clear();
        self.joint_handles.clear();

        // Create physics bodies for segments
        for segment in &self.segments {
            let handle = self.physics_world.create_segment_rigid_body(segment.pos, segment.radius);
            self.rigid_body_handles.push(handle);
        }

        // Create joints between segments
        for i in 1..self.rigid_body_handles.len() {
            let joint = self.physics_world.create_segment_joint(
                self.rigid_body_handles[i - 1],
                self.rigid_body_handles[i],
            );
            self.joint_handles.push(joint);
        }
    }

    fn update_physics(&mut self, dt: f32) {
        self.physics_world.step(dt);
    }

    fn get_rigid_body_handles(&self) -> &[RigidBodyHandle] {
        &self.rigid_body_handles
    }

    fn get_joint_handles(&self) -> &[ImpulseJointHandle] {
        &self.joint_handles
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl eframe::App for Snake {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        // UI controls in the top-left
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            self.ui.show_controls(ui, &mut self.target_segments, &mut self.show_properties, &mut self.show_skin);
        });

        // Properties panel
        if self.show_properties {
            egui::SidePanel::right("properties").show(ctx, |ui| {
                self.ui.show_properties(ui, &mut self.segments);
            });
        }

        // Main drawing area
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::drag(),
            );

            if response.rect.center() != self.center {
                self.center = response.rect.center();
                ctx.request_repaint();
            }

            if response.rect.width() > 0.0 && response.rect.height() > 0.0 {
                self.update_state(ctx);
            }

            self.draw(&painter);
        });
    }
}

impl Snake {
    pub fn add_segment(&mut self) {
        if let Some(last_segment) = self.segments.last() {
            let new_pos = last_segment.pos + self.direction * 25.0;
            let new_segment = Segment::new(
                new_pos,
                6.0,
                egui::Color32::from_rgb(100, 200, 100),
            );
            self.segments.push(new_segment.clone());
            self.target_segments += 1;

            // Create physics body for the new segment
            let rigid_body = RigidBodyBuilder::dynamic()
                .translation(vector![new_pos.x, new_pos.y])
                .linear_damping(0.3)
                .angular_damping(0.5)
                .build();
            
            let handle = self.physics_world.rigid_body_set.insert(rigid_body);
            self.rigid_body_handles.push(handle);

            // Create collider for the new segment
            let collider = ColliderBuilder::ball(new_segment.radius)
                .restitution(0.1)
                .friction(0.7)
                .build();
            
            self.physics_world.collider_set.insert_with_parent(
                collider,
                handle,
                &mut self.physics_world.rigid_body_set,
            );

            // Create joint between the new segment and the previous one
            if let Some(prev_handle) = self.rigid_body_handles.get(self.rigid_body_handles.len() - 2) {
                let joint = RevoluteJointBuilder::new()
                    .local_anchor1(point![12.0, 0.0])
                    .local_anchor2(point![-12.0, 0.0])
                    .limits([-0.5, 0.5])
                    .build();
                
                let joint_handle = self.physics_world.joint_set.insert(
                    *prev_handle,
                    handle,
                    joint,
                    true,
                );
                self.joint_handles.push(joint_handle);
            }
        }
    }
} 