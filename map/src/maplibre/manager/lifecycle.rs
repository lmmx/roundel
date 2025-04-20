// Map lifecycle management: initialization and cleanup
use crate::maplibre::bindings::Map;
use crate::maplibre::helpers::create_map_options;
use crate::utils::log::{LogCategory, with_context};
use wasm_bindgen::prelude::*;
use web_sys::window;

use super::MapLibreManager;

/// Create the actual map instance
pub fn create_map(manager: &mut MapLibreManager, container_id: &str) -> Result<(), JsValue> {
    with_context("MapLibreManager::create_map", LogCategory::Map, |logger| {
        logger.info(&format!("Creating map in container '{}'", container_id));

        // First check if maplibregl is loaded
        debug_check_maplibregl()?;

        // Create map configuration
        let options = create_map_options(container_id)?;
        logger.debug("Map options created successfully");

        // Create the map
        logger.info("Creating new Map instance");
        let map = Map::new(&options);
        logger.debug("Map instance created successfully");

        // Store the map in our manager
        manager.map = Some(map);
        logger.debug("Map stored in manager");

        // Store in window.mapInstance for compatibility with existing code
        if let Some(window) = window() {
            logger.debug("Setting window.mapInstance");
            js_sys::Reflect::set(
                &window,
                &JsValue::from_str("mapInstance"),
                &JsValue::from(manager.map.as_ref().unwrap()),
            )?;
            logger.debug("window.mapInstance set successfully");
        }

        Ok(())
    })
}

/// Clean up global map references
pub fn cleanup_map() {
    // Clear any global references
    if let Some(window) = window() {
        let _ = js_sys::Reflect::set(&window, &JsValue::from_str("mapInstance"), &JsValue::null());
    }
}

/// Debug function to check if maplibregl is available
pub fn debug_check_maplibregl() -> Result<(), JsValue> {
    with_context(
        "MapLibreManager::debug_check_maplibregl",
        LogCategory::Map,
        |logger| {
            logger.debug("Checking if maplibregl is loaded");

            let window = window().ok_or_else(|| JsValue::from_str("No window found"))?;

            // Check if maplibregl exists
            let maplibregl = js_sys::Reflect::get(&window, &JsValue::from_str("maplibregl"))?;

            if maplibregl.is_undefined() {
                logger.error("maplibregl is undefined!");
                return Err(JsValue::from_str("maplibregl is undefined"));
            }

            logger.debug("Found maplibregl object");

            // Check if Map constructor exists
            let map_constructor = js_sys::Reflect::get(&maplibregl, &JsValue::from_str("Map"))?;

            if map_constructor.is_undefined() {
                logger.error("maplibregl.Map is undefined!");
                return Err(JsValue::from_str("maplibregl.Map is undefined"));
            }

            logger.debug("Found maplibregl.Map constructor");

            // Check if it's actually a constructor/function
            if !JsValue::is_function(&map_constructor) {
                logger.error("maplibregl.Map is not a function!");
                return Err(JsValue::from_str("maplibregl.Map is not a function"));
            }

            logger.debug("maplibregl.Map is a function (constructor)");

            Ok(())
        },
    )
}
