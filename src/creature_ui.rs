use eframe::egui;

pub struct CreatureUI {
    id: String,
    creature_type: String,
}

impl CreatureUI {
    pub fn new(creature_type: &str) -> Self {
        Self {
            id: format!("{}_instance", creature_type),
            creature_type: creature_type.to_string(),
        }
    }

    pub fn show_controls(&self, ui: &mut egui::Ui, 
        target_segments: &mut usize,
        show_properties: &mut bool,
        show_skin: &mut bool,
    ) {
        ui.horizontal(|ui| {
            if ui.button(format!("{}_{}_add_segment", self.creature_type, self.id)).clicked() {
                *target_segments = (*target_segments + 1).min(20);
            }
            if ui.button(format!("{}_{}_remove_segment", self.creature_type, self.id)).clicked() {
                *target_segments = (*target_segments - 1).max(2);
            }
            ui.add(egui::DragValue::new(target_segments)
                .speed(1)
                .clamp_range(2..=20)
                .prefix("Segments: "));
            
            ui.separator();
            
            if ui.button(format!("{}_{}_toggle_properties", self.creature_type, self.id)).clicked() {
                *show_properties = !*show_properties;
            }

            ui.separator();

            if ui.button(if *show_skin { 
                format!("{}_{}_hide_skin", self.creature_type, self.id)
            } else { 
                format!("{}_{}_show_skin", self.creature_type, self.id)
            }).clicked() {
                *show_skin = !*show_skin;
            }
        });
    }

    pub fn show_properties(&self, ui: &mut egui::Ui, segments: &mut [crate::Segment]) {
        ui.heading("Segment Properties");
        ui.separator();
        
        for (i, segment) in segments.iter_mut().enumerate() {
            ui.collapsing(format!("{}_{}_segment_{}", self.creature_type, self.id, i), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Radius:");
                    ui.add(egui::DragValue::new(&mut segment.radius)
                        .speed(0.5)
                        .clamp_range(5.0..=30.0));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    let mut color = [
                        segment.color.r(),
                        segment.color.g(),
                        segment.color.b(),
                    ];
                    if ui.color_edit_button_srgb(&mut color).changed() {
                        segment.color = egui::Color32::from_rgb(
                            color[0],
                            color[1],
                            color[2],
                        );
                    }
                });
            });
        }
    }
} 