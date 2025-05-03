use eframe::egui;
use softies::creatures::{Snake};
use softies::creature::Creature;
use egui::ViewportBuilder;

pub struct CreatureApp {
    current_creature: Box<dyn Creature>,
    creature_type: String,
    show_properties: bool,
}

impl Default for CreatureApp {
    fn default() -> Self {
        Self {
            current_creature: Box::new(Snake::default()),
            creature_type: "test_chain".to_string(),
            show_properties: false,
        }
    }
}

impl eframe::App for CreatureApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Draw the current creature
        self.current_creature.update_state(ctx);
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Background,
            egui::Id::new("creature"),
        ));
        self.current_creature.draw(&painter);

        // Draw UI
        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.heading("Controls");
            if ui.button("Snake").clicked() {
                self.current_creature = Box::new(Snake::default());
                self.creature_type = "snake".to_string();
            }
            ui.separator();
            if ui.button("Toggle Properties").clicked() {
                self.show_properties = !self.show_properties;
            }
        });

        if self.show_properties {
            egui::SidePanel::right("properties").show(ctx, |ui| {
                ui.heading("Properties");
                self.current_creature.show_properties(ui);
            });
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Softies",
        options,
        Box::new(|_cc| Box::new(CreatureApp::default())),
    )
}
