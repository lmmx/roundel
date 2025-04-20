use dioxus::prelude::*;
use wasm_bindgen::closure::Closure;

use super::TflLayers;
use crate::data::TflDataRepository;
use crate::maplibre::helpers::{add_inline_script, load_css, load_script};
use crate::maplibre::manager::MapLibreManager;
use crate::utils::log::{self, LogCategory};

#[component]
pub fn Canvas(layers: Signal<TflLayers>, tfl_data: Signal<TflDataRepository>) -> Element {
    // Add a flag to track if we've already initialized the map
    let mut already_initialized = use_signal(|| false);

    // 1) A Dioxus state handle for your manager
    let manager = use_signal(|| {
        log::info_with_category(LogCategory::Map, "Creating new MapLibreManager");
        MapLibreManager::new()
    });

    // 2) Run this effect only once during mount
    // To avoid the infinite loop, we won't read and write to the same signal
    use_effect(move || {
        log::info_with_category(LogCategory::Map, "Canvas effect starting");

        // Check if we've already initialized - avoid double initialization
        if *already_initialized.write() {
            log::info_with_category(LogCategory::Map, "Map already initialized, skipping");
            return {};
        }

        // Mark as initialized immediately to prevent potential recursion
        already_initialized.set(true);

        // Load any CSS or inline scripts - these run once since they're in a use_effect with no dependencies
        log::info_with_category(LogCategory::Map, "Loading CSS files...");
        let _ = load_css("https://unpkg.com/maplibre-gl@3.6.2/dist/maplibre-gl.css");

        log::info_with_category(LogCategory::Map, "Loading JS controls...");
        let _ = add_inline_script(include_str!("../app/js/key_control.js"));
        let _ = add_inline_script(include_str!("../app/js/layer_switcher.js"));
        let _ = add_inline_script(include_str!("../app/js/simulation_control.js"));

        // Prepare the "on_load" closure for when the external script finishes
        let mut manager_clone = manager; // Create a clone to avoid capturing the original signal

        log::info_with_category(LogCategory::Map, "Creating script onload closure...");
        let on_load = Closure::wrap(Box::new(move || {
            log::info_with_category(
                LogCategory::Map,
                "MapLibre script loaded callback executing",
            );

            let mg = &mut manager_clone.write();
            log::info_with_category(LogCategory::Map, "Creating map...");
            if let Err(err) = mg.create_map("maplibre-canvas") {
                log::error_with_category(
                    LogCategory::Map,
                    &format!("Failed to create map: {err:?}"),
                );
                return;
            }

            log::info_with_category(LogCategory::Map, "Adding map controls...");
            if let Err(err) = mg.add_map_controls() {
                log::error_with_category(
                    LogCategory::Map,
                    &format!("Failed to add map controls: {err:?}"),
                );
                return;
            }

            log::info_with_category(LogCategory::Map, "Setting up map data...");
            if let Err(err) = mg.setup_map_data(layers.read().simulation, tfl_data.read().clone()) {
                log::error_with_category(
                    LogCategory::Map,
                    &format!("Failed to set up map data: {err:?}"),
                );
                return;
            }

            log::info_with_category(
                LogCategory::Map,
                "Map initialization completed successfully",
            );
        }) as Box<dyn FnMut()>);

        // Load the main MapLibre script and pass our closure
        log::info_with_category(LogCategory::Map, "Loading MapLibre script...");
        let script_result = load_script(
            "https://unpkg.com/maplibre-gl@3.6.2/dist/maplibre-gl.js",
            Some(on_load),
        );

        if let Err(err) = script_result {
            log::error_with_category(
                LogCategory::Map,
                &format!("Failed to load MapLibre script: {:?}", err),
            );
        }

        log::info_with_category(LogCategory::Map, "Canvas effect setup completed");

        // Return an empty cleanup closure
        {
            log::info_with_category(LogCategory::Map, "Canvas effect cleanup called");
        }
    });

    // 3) Render the container in your JSX/RSX
    rsx! {
        div {
            id: "map-container",
            style: "position:relative; width:100%; height:100vh;",
            div {
                id: "maplibre-canvas",
                style: "position:absolute; top:0; bottom:0; width:100%;"
            }
        }
    }
}
