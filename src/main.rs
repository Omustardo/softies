use softies::app::SoftiesApp; 

// Keep module declarations, but main doesn't use them directly
mod creature;
mod creatures;
mod creature_attributes; // Re-enable this module for the binary crate

// Constants for the aquarium
#[allow(dead_code)]
const AQUARIUM_WIDTH: f32 = 500.0;
#[allow(dead_code)]
const AQUARIUM_HEIGHT: f32 = 300.0;
#[allow(dead_code)]
const MIN_ZOOM: f32 = 0.1;
#[allow(dead_code)]
const MAX_ZOOM: f32 = 5.0;
#[allow(dead_code)]
const CAMERA_BOUND_PADDING: f32 = 0.3; // 30% padding

fn main() -> eframe::Result<()> {
    // Setup tracing for native panic info with more verbose output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Softies Aquarium",
        native_options,
        Box::new(|_cc| Box::new(SoftiesApp::default())),
    )
}
