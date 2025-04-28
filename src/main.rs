use eframe::egui;
use softies::{SpiralCreature, Snake};

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
}

impl Default for CreatureApp {
    fn default() -> Self {
        Self {
            current_creature: Box::new(SpiralCreature::default()),
        }
    }
}

impl eframe::App for CreatureApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // UI controls in the top-left
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Spiral Creature").clicked() {
                    self.current_creature = Box::new(SpiralCreature::default());
                }
                if ui.button("Snake Creature").clicked() {
                    self.current_creature = Box::new(Snake::default());
                }
            });
        });

        // Delegate to the current creature
        self.current_creature.update(ctx, frame);
    }
}
