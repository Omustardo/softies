pub mod creature;
pub mod creature_ui;
pub mod creatures;

pub use creature::{Creature, Segment};
pub use creature_ui::CreatureUI;
pub use creatures::{TestChain};

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    let web_options = eframe::WebOptions::default();
    
    eframe::WebRunner::new()
        .start(
            canvas_id,
            web_options,
            Box::new(|_cc| Box::new(TestChain::default())),
        )
        .await
} 