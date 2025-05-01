use eframe::egui;

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