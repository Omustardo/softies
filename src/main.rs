use eframe::egui;
use softies::{DemoCreature, Snake};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Creature Demo",
        options,
        Box::new(|_cc| Box::new(CreatureApp::default())),
    )
}

struct CreatureApp {
    current_creature: Box<dyn eframe::App>,
    creature_type: String,
    show_properties: bool,
    show_skin: bool,
}

impl Default for CreatureApp {
    fn default() -> Self {
        Self {
            current_creature: Box::new(DemoCreature::default()),
            creature_type: "demo".to_string(),
            show_properties: false,
            show_skin: true,
        }
    }
}

impl eframe::App for CreatureApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // UI controls in the top-left
        egui::TopBottomPanel::top("creature_controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let demo_response = ui.button("Demo Creature");
                if demo_response.clicked() {
                    self.current_creature = Box::new(DemoCreature::default());
                    self.creature_type = "demo".to_string();
                }
                
                let snake_response = ui.button("Snake Creature");
                if snake_response.clicked() {
                    self.current_creature = Box::new(Snake::default());
                    self.creature_type = "snake".to_string();
                }

                ui.separator();

                if ui.button(if self.show_properties { "Hide Properties" } else { "Show Properties" }).clicked() {
                    self.show_properties = !self.show_properties;
                }

                if ui.button(if self.show_skin { "Hide Skin" } else { "Show Skin" }).clicked() {
                    self.show_skin = !self.show_skin;
                }
            });
        });

        // Properties panel
        if self.show_properties {
            egui::SidePanel::right("properties_panel").show(ctx, |ui| {
                ui.heading("Properties");
                ui.separator();
                // Add property controls here
            });
        }

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Delegate to the current creature
            self.current_creature.update(ctx, frame);
        });
    }
}
