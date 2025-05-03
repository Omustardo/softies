pub mod app;
pub mod creature;
pub mod creatures;

// Segment is defined in creature.rs (Bevy version), remove direct export for now
// pub use creature::{Creature, Segment};

// Restore import needed for wasm build
use crate::app::SoftiesApp;

// Remove unused import
// use crate::app::SoftiesApp;

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    
    eframe::WebRunner::new()
        .start(
            canvas_id,
            web_options,
            Box::new(|_cc| Box::new(SoftiesApp::default())), // Use SoftiesApp
        )
        .await
} 