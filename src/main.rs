use eframe::egui;
use softies::creatures::{DemoCreature, Snake, TestChain, SimpleChain, Plankton};
use softies::creature::Creature;
use rand::Rng;
use egui::ViewportBuilder;

pub struct CreatureApp {
    current_creature: Box<dyn Creature>,
    creature_type: String,
    show_properties: bool,
    plankton: Option<Plankton>,
}

impl Default for CreatureApp {
    fn default() -> Self {
        Self {
            current_creature: Box::new(SimpleChain::default()),
            creature_type: "simple".to_string(),
            show_properties: false,
            plankton: None,
        }
    }
}

impl eframe::App for CreatureApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update creature type if needed
        if let Some(_snake) = self.current_creature.as_any().downcast_ref::<Snake>() {
            if self.creature_type != "snake" {
                self.current_creature = Box::new(Snake::default());
                self.creature_type = "snake".to_string();
            }
        }

        // Draw the current creature
        self.current_creature.update_state(ctx);
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Background,
            egui::Id::new("creature"),
        ));
        self.current_creature.draw(&painter);

        // Handle plankton
        if self.creature_type == "snake" {
            // Initialize plankton if needed
            if self.plankton.is_none() {
                let pos = egui::Pos2::new(400.0, 300.0);
                self.plankton = Some(Plankton::new(pos));
            }

            // Update and draw plankton
            if let Some(plankton) = &mut self.plankton {
                plankton.update_state(ctx);
                plankton.draw(&painter);

                // Check for collision with snake
                if let Some(snake) = self.current_creature.as_any().downcast_ref::<Snake>() {
                    if let Some(plankton) = &self.plankton {
                        if !plankton.is_eaten() {
                            let snake_pos = snake.get_segments()[0].pos;
                            let plankton_pos = plankton.get_position();
                            let distance = (snake_pos - plankton_pos).length();
                            
                            if distance < snake.get_segments()[0].radius + plankton.get_segments()[0].radius {
                                // Snake ate the plankton
                                if let Some(snake) = self.current_creature.as_any().downcast_ref::<Snake>() {
                                    let mut new_snake = snake.clone();
                                    new_snake.add_segment();
                                    self.current_creature = Box::new(new_snake);
                                }
                                if let Some(plankton) = &mut self.plankton {
                                    plankton.mark_as_eaten();
                                    // Respawn plankton at random position
                                    let mut rng = rand::thread_rng();
                                    let x = rng.gen_range(100.0..700.0);
                                    let y = rng.gen_range(100.0..500.0);
                                    plankton.respawn(egui::Pos2::new(x, y));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Draw UI
        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.heading("Controls");
            if ui.button("Simple Chain").clicked() {
                self.current_creature = Box::new(SimpleChain::default());
                self.creature_type = "simple".to_string();
                self.plankton = None;
            }
            if ui.button("Demo Creature").clicked() {
                self.current_creature = Box::new(DemoCreature::default());
                self.creature_type = "demo".to_string();
                self.plankton = None;
            }
            if ui.button("Snake Creature").clicked() {
                self.current_creature = Box::new(Snake::default());
                self.creature_type = "snake".to_string();
                // Initialize plankton when switching to snake
                let pos = egui::Pos2::new(400.0, 300.0);
                self.plankton = Some(Plankton::new(pos));
            }
            if ui.button("Test Chain").clicked() {
                self.current_creature = Box::new(TestChain::default());
                self.creature_type = "test".to_string();
                self.plankton = None;
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
