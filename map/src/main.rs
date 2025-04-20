use dioxus::prelude::*;

// For logging and better errors in WASM
use log::Level;

mod app;
mod data;
mod maplibre; // Add the new MapLibre module
mod tb8;
mod utils; // Add the new utils module

use utils::{LogLevel, set_log_level};

/// Entry point for Dioxus
fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Info).expect("error initializing logger");

    // Initialize our custom logger
    #[cfg(debug_assertions)]
    {
        // Set to Debug level in development
        set_log_level(LogLevel::Debug);
    }
    #[cfg(not(debug_assertions))]
    {
        // Set to Info level in production
        set_log_level(LogLevel::Info);
    }

    utils::log::info("Application starting...");

    launch(app::app)
}
