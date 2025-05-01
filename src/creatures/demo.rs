use eframe::egui;
use rapier2d::prelude::*;
use crate::{creature::{Creature, Segment, PhysicsWorld}, creature_ui::CreatureUI};
use std::any::Any;

const PIXELS_PER_METER: f32 = 50.0;

pub struct DemoCreature {
    segments: Vec<Segment>,
    target_segments: usize,
    show_properties: bool,
    show_skin: bool,
    time: f32,
    center: egui::Pos2,
    ui: CreatureUI,
    target_pos: egui::Pos2,
    speed: f32,
    
    // Physics components
    physics_world: PhysicsWorld,
    rigid_body_handles: Vec<RigidBodyHandle>,
    joint_handles: Vec<ImpulseJointHandle>,
    startup_delay: f32,

    // Physics parameters
    linear_damping: f32,
    angular_damping: f32,
    joint_limits: f32,
    motor_stiffness: f32,
    motor_damping: f32,
    head_speed: f32,
    body_speed: f32,
    spring_constant: f32,
}

impl Default for DemoCreature {
    fn default() -> Self {
        let mut segments = Vec::new();
        let start_pos = egui::Pos2::new(400.0, 300.0);
        let mut current_pos = start_pos;
        let direction = egui::Vec2::new(-1.0, 0.0);  // Start moving left

        // Create initial segments
        for i in 0..8 {
            segments.push(Segment::new(
                current_pos,
                if i == 0 { 15.0 } else { 10.0 },  // Larger head
                if i == 0 {
                    egui::Color32::from_rgb(200, 100, 100)  // Red for head
                } else {
                    egui::Color32::from_rgb(100, 200, 100)  // Green for body
                },
            ));
            current_pos = current_pos + direction * 30.0;  // Reduced spacing for tighter chain
        }

        let mut physics_world = PhysicsWorld::default();
        let mut rigid_body_handles = Vec::new();
        let mut joint_handles = Vec::new();

        // Create physics bodies for initial segments
        for (i, segment) in segments.iter().enumerate() {
            // Convert from pixels to meters for physics
            let pos_meters = vector![
                segment.pos.x / PIXELS_PER_METER,
                segment.pos.y / PIXELS_PER_METER
            ];
            let radius_meters = segment.radius / PIXELS_PER_METER;

            let rigid_body = RigidBodyBuilder::dynamic()
                .translation(pos_meters)
                .linear_damping(0.98)  // Increased damping for stability
                .angular_damping(0.98)  // Increased damping for stability
                .dominance_group(if i == 0 { 1 } else { 0 })
                .build();
            
            let handle = physics_world.rigid_body_set.insert(rigid_body);
            rigid_body_handles.push(handle);

            // Create collider
            let collider = ColliderBuilder::ball(radius_meters)
                .restitution(0.1)
                .friction(0.3)
                .build();
            
            physics_world.collider_set.insert_with_parent(
                collider,
                handle,
                &mut physics_world.rigid_body_set,
            );
        }

        // Create joints with improved parameters
        let target_distance = 30.0 / PIXELS_PER_METER;  // Convert 30 pixels to meters
        for i in 1..rigid_body_handles.len() {
            let joint = FixedJointBuilder::new()
                .local_frame1(Isometry::translation(0.0, 0.0))
                .local_frame2(Isometry::translation(target_distance, 0.0))
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
            ui: CreatureUI::new("demo"),
            target_pos: start_pos,
            speed: 300.0,
            physics_world,
            rigid_body_handles,
            joint_handles,
            startup_delay: 1.0,

            // Default physics parameters from successful test
            linear_damping: 0.98,
            angular_damping: 0.98,
            joint_limits: 0.1,
            motor_stiffness: 0.0,  // Not used with fixed joints
            motor_damping: 0.0,    // Not used with fixed joints
            head_speed: 1.0,       // Reduced for smoother movement
            body_speed: 0.8,       // Reduced for smoother movement
            spring_constant: 0.0,  // Not used with fixed joints
        }
    }
}

impl DemoCreature {
    fn reset_physics(&mut self) {
        // Clear existing physics objects
        self.physics_world = PhysicsWorld::default();
        self.rigid_body_handles.clear();
        self.joint_handles.clear();

        // Reset positions
        let start_pos = egui::Pos2::new(400.0, 300.0);
        let mut current_pos = start_pos;
        let direction = egui::Vec2::new(-1.0, 0.0);

        // Reset segments
        for segment in &mut self.segments {
            segment.pos = current_pos;
            current_pos = current_pos + direction * 30.0;
        }

        // Create physics bodies with high damping for stability
        for (i, segment) in self.segments.iter().enumerate() {
            let pos_meters = vector![
                segment.pos.x / PIXELS_PER_METER,
                segment.pos.y / PIXELS_PER_METER
            ];
            let radius_meters = segment.radius / PIXELS_PER_METER;

            let rigid_body = RigidBodyBuilder::dynamic()
                .translation(pos_meters)
                .linear_damping(self.linear_damping)
                .angular_damping(self.angular_damping)
                .dominance_group(if i == 0 { 1 } else { 0 })
                .build();
            
            let handle = self.physics_world.rigid_body_set.insert(rigid_body);
            self.rigid_body_handles.push(handle);

            let collider = ColliderBuilder::ball(radius_meters)
                .restitution(0.1)
                .friction(0.3)
                .build();
            
            self.physics_world.collider_set.insert_with_parent(
                collider,
                handle,
                &mut self.physics_world.rigid_body_set,
            );
        }

        // Create distance joints between segments
        let target_distance = 30.0 / PIXELS_PER_METER;  // Convert 30 pixels to meters
        for i in 1..self.rigid_body_handles.len() {
            let joint = FixedJointBuilder::new()
                .local_frame1(Isometry::translation(0.0, 0.0))
                .local_frame2(Isometry::translation(target_distance, 0.0))
                .build();
            
            let handle = self.physics_world.joint_set.insert(
                self.rigid_body_handles[i - 1],
                self.rigid_body_handles[i],
                joint,
                true,
            );
            self.joint_handles.push(handle);
        }

        // Reset startup delay
        self.startup_delay = 1.0;
    }
}

impl Creature for DemoCreature {
    fn update_state(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.unstable_dt);
        if dt > 0.0 {
            self.time += dt;
            self.startup_delay -= dt;

            // Only apply motion after startup delay
            if self.startup_delay <= 0.0 {
                // Get cursor position in physics units
                let cursor_pos = ctx.pointer_interact_pos()
                    .map(|pos| vector![
                        pos.x / PIXELS_PER_METER,
                        pos.y / PIXELS_PER_METER
                    ]);

                if let Some(cursor_pos) = cursor_pos {
                    // Update head movement
                    if let Some(head_handle) = self.rigid_body_handles.first() {
                        if let Some(head) = self.physics_world.rigid_body_set.get_mut(*head_handle) {
                            let head_pos = head.translation();
                            let to_cursor = cursor_pos - head_pos;
                            let distance = to_cursor.magnitude();

                            if distance > 0.1 {
                                // Calculate desired velocity towards cursor
                                let direction = to_cursor / distance;
                                let target_speed = self.head_speed.min(distance * 2.0); // Scale speed with distance
                                let velocity = direction * target_speed;

                                // Smoothly adjust current velocity
                                let current_vel = head.linvel();
                                let new_vel = current_vel + (velocity - current_vel) * 0.1; // Smooth acceleration
                                head.set_linvel(new_vel, true);

                                // Add slight rotation to face movement direction
                                let angle = to_cursor.y.atan2(to_cursor.x);
                                let current_angle = head.rotation().angle();
                                let new_angle = current_angle + (angle - current_angle) * 0.1; // Smooth rotation
                                head.set_rotation(Rotation::new(new_angle), true);
                            }
                        }
                    }

                    // Update body segments to follow their predecessors
                    // First collect all positions
                    let mut positions = Vec::with_capacity(self.rigid_body_handles.len());
                    for handle in &self.rigid_body_handles {
                        if let Some(body) = self.physics_world.rigid_body_set.get(*handle) {
                            positions.push(*body.translation());
                        } else {
                            positions.push(vector![0.0, 0.0]);
                        }
                    }

                    // Then update each body using the collected positions
                    for i in 1..self.rigid_body_handles.len() {
                        if let Some(curr_body) = self.physics_world.rigid_body_set.get_mut(self.rigid_body_handles[i]) {
                            let prev_pos = positions[i - 1];
                            let curr_pos = positions[i];
                            let to_prev = prev_pos - curr_pos;
                            let distance = to_prev.magnitude();
                            
                            if distance > 0.1 {
                                // Calculate follow velocity
                                let direction = to_prev / distance;
                                let target_speed = self.body_speed.min(distance * 2.0); // Scale speed with distance
                                let velocity = direction * target_speed;

                                // Smoothly adjust current velocity
                                let current_vel = curr_body.linvel();
                                let new_vel = current_vel + (velocity - current_vel) * 0.1; // Smooth acceleration
                                curr_body.set_linvel(new_vel, true);
                            }
                        }
                    }
                }
            }

            // Step physics with a fixed timestep for stability
            self.physics_world.step(1.0/60.0);

            // Update segment positions
            for (i, handle) in self.rigid_body_handles.iter().enumerate() {
                if let Some(body) = self.physics_world.rigid_body_set.get(*handle) {
                    let pos = body.translation();
                    // Convert from meters back to pixels
                    self.segments[i].pos = egui::Pos2::new(
                        pos.x * PIXELS_PER_METER,
                        pos.y * PIXELS_PER_METER
                    );
                    
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

            // Request continuous repaint for smooth animation
            ctx.request_repaint();
        }

        // Adjust number of segments if needed
        while self.segments.len() < self.target_segments {
            let last_pos = self.segments.last().unwrap().pos;
            let direction = if self.segments.len() > 1 {
                (self.segments[self.segments.len()-2].pos - last_pos).normalized()
            } else {
                egui::Vec2::new(-1.0, 0.0)
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
        "Demo"
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

    fn show_properties(&mut self, ui: &mut egui::Ui) {
        ui.heading("Physics Parameters");
        
        // Add reset button
        if ui.button("Reset to Defaults").clicked() {
            self.linear_damping = 0.9;
            self.angular_damping = 0.9;
            self.joint_limits = 0.2;
            self.motor_stiffness = 5.0;
            self.motor_damping = 0.8;
            self.head_speed = 2.0;
            self.body_speed = 1.5;
            self.spring_constant = 5.0;
            self.reset_physics();
        }

        // Add copy button
        if ui.button("Copy Values to Clipboard").clicked() {
            let values = format!(
                "linear_damping: {:.2}\nangular_damping: {:.2}\njoint_limits: {:.2}\nmotor_stiffness: {:.2}\nmotor_damping: {:.2}\nhead_speed: {:.2}\nbody_speed: {:.2}\nspring_constant: {:.2}",
                self.linear_damping,
                self.angular_damping,
                self.joint_limits,
                self.motor_stiffness,
                self.motor_damping,
                self.head_speed,
                self.body_speed,
                self.spring_constant
            );
            ui.output_mut(|o| o.copied_text = values);
        }

        ui.separator();

        // Add sliders for each parameter
        let mut changed = false;
        
        changed |= ui.add(egui::Slider::new(&mut self.linear_damping, 0.0..=1.0)
            .text("Linear Damping")).changed();
        
        changed |= ui.add(egui::Slider::new(&mut self.angular_damping, 0.0..=1.0)
            .text("Angular Damping")).changed();
        
        changed |= ui.add(egui::Slider::new(&mut self.joint_limits, 0.1..=0.5)
            .text("Joint Limits")).changed();
        
        changed |= ui.add(egui::Slider::new(&mut self.motor_stiffness, 1.0..=20.0)
            .text("Motor Stiffness")).changed();
        
        changed |= ui.add(egui::Slider::new(&mut self.motor_damping, 0.1..=1.0)
            .text("Motor Damping")).changed();
        
        changed |= ui.add(egui::Slider::new(&mut self.head_speed, 1.0..=5.0)
            .text("Head Speed")).changed();
        
        changed |= ui.add(egui::Slider::new(&mut self.body_speed, 0.5..=3.0)
            .text("Body Speed")).changed();
        
        changed |= ui.add(egui::Slider::new(&mut self.spring_constant, 1.0..=20.0)
            .text("Spring Constant")).changed();

        // Reset physics if any parameter changed
        if changed {
            self.reset_physics();
        }

        ui.separator();
        ui.heading("Segment Properties");
        for (i, segment) in self.segments.iter_mut().enumerate() {
            ui.collapsing(format!("Segment {}", i), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Radius:");
                    ui.add(egui::DragValue::new(&mut segment.radius)
                        .speed(1.0)
                        .clamp_range(5.0..=20.0));
                });
            });
        }
    }
}

impl eframe::App for DemoCreature {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        // UI controls in the top-left
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Properties").clicked() {
                    self.show_properties = !self.show_properties;
                    println!("Properties toggled to: {}", self.show_properties); // Debug print
                }
                if ui.button("Show Skin").clicked() {
                    self.show_skin = !self.show_skin;
                }
                ui.label("Target Segments:");
                ui.add(egui::DragValue::new(&mut self.target_segments)
                    .speed(1)
                    .clamp_range(1..=20));
            });
        });

        // Properties panel
        if self.show_properties {
            egui::SidePanel::right("properties").show(ctx, |ui| {
                self.show_properties(ui);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_parameters() {
        // Test parameters that focus on stability without movement
        let params = PhysicsParams {
            linear_damping: 0.99,  // Very high damping
            angular_damping: 0.99,  // Very high damping
            joint_limits: 0.1,      // Not used with distance joints
            motor_stiffness: 0.0,   // Not used with distance joints
            motor_damping: 0.0,     // Not used with distance joints
            head_speed: 0.0,
            body_speed: 0.0,
            spring_constant: 0.0,    // Not used with distance joints
        };

        let mut creature = DemoCreature::default();
        
        // Apply test parameters
        creature.linear_damping = params.linear_damping;
        creature.angular_damping = params.angular_damping;
        creature.joint_limits = params.joint_limits;
        creature.motor_stiffness = params.motor_stiffness;
        creature.motor_damping = params.motor_damping;
        creature.head_speed = params.head_speed;
        creature.body_speed = params.body_speed;
        creature.spring_constant = params.spring_constant;
        
        // Reset physics with new parameters
        creature.reset_physics();

        // Record initial positions before simulation
        let mut initial_positions = Vec::new();
        for handle in &creature.rigid_body_handles {
            if let Some(body) = creature.physics_world.rigid_body_set.get(*handle) {
                initial_positions.push(*body.translation());
            }
        }

        // Simulate for 1 second with fixed timestep
        let dt = 1.0 / 60.0;
        let mut time = 0.0;
        let max_time = 1.0;
        let mut is_stable = true;
        let mut max_velocity: f32 = 0.0;
        let mut max_displacement: f32 = 0.0;

        while time < max_time {
            // Step physics
            creature.physics_world.step(dt);

            // Check positions and velocities of all segments
            for (i, handle) in creature.rigid_body_handles.iter().enumerate() {
                if let Some(body) = creature.physics_world.rigid_body_set.get(*handle) {
                    // Check velocity
                    let vel = body.linvel();
                    let speed = (vel.x * vel.x + vel.y * vel.y).sqrt();
                    max_velocity = max_velocity.max(speed);

                    // Check displacement from initial position
                    let pos = *body.translation();
                    let initial_pos = initial_positions[i];
                    let displacement = (pos - initial_pos).magnitude();
                    max_displacement = max_displacement.max(displacement);

                    // Convert position from meters to pixels
                    let pos_pixels = egui::Pos2::new(
                        pos.x * PIXELS_PER_METER,
                        pos.y * PIXELS_PER_METER
                    );

                    // Check if out of bounds
                    if pos_pixels.x < -100.0 || pos_pixels.x > 900.0 ||
                       pos_pixels.y < -100.0 || pos_pixels.y > 700.0 {
                        is_stable = false;
                        break;
                    }
                }
            }

            if !is_stable {
                break;
            }

            time += dt;
        }

        println!("\nTest Results:");
        println!("Stable: {}", is_stable);
        println!("Max Velocity: {:.2} m/s", max_velocity);
        println!("Max Displacement: {:.2} m", max_displacement);

        // A successful test means:
        // 1. The creature stays in bounds
        // 2. Velocities remain small (indicating good damping)
        // 3. Displacement is small (indicating good joint constraints)
        assert!(is_stable, "Creature went out of bounds");
        assert!(max_velocity < 1.0, "Velocity too high: {:.2} m/s", max_velocity);
        assert!(max_displacement < 0.5, "Displacement too high: {:.2} m", max_displacement);
    }

    #[test]
    fn test_movement_with_fixed_joints() {
        // Test parameters for movement
        let params = PhysicsParams {
            linear_damping: 0.98,    // Higher damping for stability
            angular_damping: 0.98,    // Higher damping for stability
            joint_limits: 0.1,        // Not used with fixed joints
            motor_stiffness: 0.0,     // Not used with fixed joints
            motor_damping: 0.0,       // Not used with fixed joints
            head_speed: 1.0,          // Reduced head speed
            body_speed: 0.8,          // Reduced body speed
            spring_constant: 0.0,     // Not used with fixed joints
        };

        let mut creature = DemoCreature::default();
        
        // Apply test parameters
        creature.linear_damping = params.linear_damping;
        creature.angular_damping = params.angular_damping;
        creature.joint_limits = params.joint_limits;
        creature.motor_stiffness = params.motor_stiffness;
        creature.motor_damping = params.motor_damping;
        creature.head_speed = params.head_speed;
        creature.body_speed = params.body_speed;
        creature.spring_constant = params.spring_constant;
        
        // Reset physics with new parameters
        creature.reset_physics();

        // Record initial positions
        let mut initial_positions = Vec::new();
        for handle in &creature.rigid_body_handles {
            if let Some(body) = creature.physics_world.rigid_body_set.get(*handle) {
                initial_positions.push(*body.translation());
            }
        }

        // Simulate for 2 seconds with fixed timestep
        let dt = 1.0 / 60.0;
        let mut time = 0.0;
        let max_time = 2.0;
        let mut is_stable = true;
        let mut max_velocity: f32 = 0.0;
        let mut max_displacement: f32 = 0.0;
        let mut total_movement: f32 = 0.0;

        // Create a test context that moves the cursor in a figure-8 pattern
        let mut test_ctx = TestContext::new();

        while time < max_time {
            // Update cursor position
            test_ctx.update(dt);
            
            // Get cursor position in physics units
            let cursor_pos = vector![
                test_ctx.cursor_pos.x / PIXELS_PER_METER,
                test_ctx.cursor_pos.y / PIXELS_PER_METER
            ];

            // Update head movement
            if let Some(head_handle) = creature.rigid_body_handles.first() {
                if let Some(head) = creature.physics_world.rigid_body_set.get_mut(*head_handle) {
                    let head_pos = head.translation();
                    let to_cursor = cursor_pos - head_pos;
                    let distance = to_cursor.magnitude();

                    if distance > 0.1 {
                        // Calculate desired velocity towards cursor
                        let direction = to_cursor / distance;
                        let target_speed = params.head_speed.min(distance * 2.0); // Scale speed with distance
                        let velocity = direction * target_speed;

                        // Smoothly adjust current velocity
                        let current_vel = head.linvel();
                        let new_vel = current_vel + (velocity - current_vel) * 0.1; // Smooth acceleration
                        head.set_linvel(new_vel, true);

                        // Add slight rotation to face movement direction
                        let angle = to_cursor.y.atan2(to_cursor.x);
                        let current_angle = head.rotation().angle();
                        let new_angle = current_angle + (angle - current_angle) * 0.1; // Smooth rotation
                        head.set_rotation(Rotation::new(new_angle), true);
                    }
                }
            }

            // Update body segments to follow their predecessors
            let mut positions = Vec::with_capacity(creature.rigid_body_handles.len());
            for handle in &creature.rigid_body_handles {
                if let Some(body) = creature.physics_world.rigid_body_set.get(*handle) {
                    positions.push(*body.translation());
                } else {
                    positions.push(vector![0.0, 0.0]);
                }
            }

            for i in 1..creature.rigid_body_handles.len() {
                if let Some(curr_body) = creature.physics_world.rigid_body_set.get_mut(creature.rigid_body_handles[i]) {
                    let prev_pos = positions[i - 1];
                    let curr_pos = positions[i];
                    let to_prev = prev_pos - curr_pos;
                    let distance = to_prev.magnitude();
                    
                    if distance > 0.1 {
                        // Calculate follow velocity
                        let direction = to_prev / distance;
                        let target_speed = params.body_speed.min(distance * 2.0); // Scale speed with distance
                        let velocity = direction * target_speed;

                        // Smoothly adjust current velocity
                        let current_vel = curr_body.linvel();
                        let new_vel = current_vel + (velocity - current_vel) * 0.1; // Smooth acceleration
                        curr_body.set_linvel(new_vel, true);
                    }
                }
            }

            // Step physics
            creature.physics_world.step(dt);

            // Check positions and velocities
            for (i, handle) in creature.rigid_body_handles.iter().enumerate() {
                if let Some(body) = creature.physics_world.rigid_body_set.get(*handle) {
                    // Check velocity
                    let vel = body.linvel();
                    let speed = (vel.x * vel.x + vel.y * vel.y).sqrt();
                    max_velocity = max_velocity.max(speed);

                    // Check displacement from initial position
                    let pos = *body.translation();
                    let initial_pos = initial_positions[i];
                    let displacement = (pos - initial_pos).magnitude();
                    max_displacement = max_displacement.max(displacement);

                    // Track total movement
                    if i == 0 {  // Only track head movement
                        total_movement += speed * dt;
                    }

                    // Convert position from meters to pixels
                    let pos_pixels = egui::Pos2::new(
                        pos.x * PIXELS_PER_METER,
                        pos.y * PIXELS_PER_METER
                    );

                    // Check if out of bounds
                    if pos_pixels.x < -100.0 || pos_pixels.x > 900.0 ||
                       pos_pixels.y < -100.0 || pos_pixels.y > 700.0 {
                        is_stable = false;
                        break;
                    }
                }
            }

            if !is_stable {
                break;
            }

            time += dt;
        }

        println!("\nMovement Test Results:");
        println!("Stable: {}", is_stable);
        println!("Max Velocity: {:.2} m/s", max_velocity);
        println!("Max Displacement: {:.2} m", max_displacement);
        println!("Total Head Movement: {:.2} m", total_movement);

        // A successful movement test means:
        // 1. The creature stays in bounds
        // 2. Velocities remain reasonable
        // 3. The head moves a significant distance
        // 4. The body follows the head without excessive stretching
        assert!(is_stable, "Creature went out of bounds");
        assert!(max_velocity < 3.0, "Velocity too high: {:.2} m/s", max_velocity);
        assert!(max_displacement < 2.0, "Displacement too high: {:.2} m", max_displacement);
        assert!(total_movement > 1.0, "Insufficient movement: {:.2} m", total_movement);
    }

    // Helper struct to simulate cursor movement
    struct TestContext {
        cursor_pos: egui::Pos2,
        time: f32,
    }

    impl TestContext {
        fn new() -> Self {
            Self {
                cursor_pos: egui::Pos2::new(400.0, 300.0),
                time: 0.0,
            }
        }

        fn update(&mut self, dt: f32) {
            self.time += dt;
            // Move cursor in a figure-8 pattern
            let scale = 200.0;
            let speed = 1.0;  // Reduced speed for smoother movement
            self.cursor_pos.x = 400.0 + scale * (self.time * speed).sin();
            self.cursor_pos.y = 300.0 + scale * (self.time * speed * 2.0).sin();
        }
    }

    #[derive(Clone, Copy)]
    struct PhysicsParams {
        linear_damping: f32,
        angular_damping: f32,
        joint_limits: f32,
        motor_stiffness: f32,
        motor_damping: f32,
        head_speed: f32,
        body_speed: f32,
        spring_constant: f32,
    }
} 