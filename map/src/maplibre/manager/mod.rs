// Main module file that re-exports and provides the core Manager struct
mod controls;
mod events;
mod layers;
mod lifecycle;

pub use controls::*;
pub use events::*;
pub use layers::*;

use crate::data::TflDataRepository;
use crate::maplibre::bindings::Map;
use crate::utils::log::{self, LogCategory};
use wasm_bindgen::prelude::*;

/// Main MapLibre manager that coordinates components
pub struct MapLibreManager {
    pub map: Option<Map>,
    event_manager: EventManager,
    layer_manager: LayerManager,
    control_manager: ControlManager,
}

impl MapLibreManager {
    /// Create a new manager (without initializing the map yet)
    pub fn new() -> Self {
        log::info_with_category(LogCategory::Map, "MapLibreManager::new() called");
        Self {
            map: None,
            event_manager: EventManager::new(),
            layer_manager: LayerManager::new(),
            control_manager: ControlManager::new(),
        }
    }

    /// Create the actual map instance
    pub fn create_map(&mut self, container_id: &str) -> Result<(), JsValue> {
        // Delegate to lifecycle module
        lifecycle::create_map(self, container_id)
    }

    /// Check if maplibregl is available (static debug function)
    pub fn debug_check_maplibregl() -> Result<(), JsValue> {
        lifecycle::debug_check_maplibregl()
    }

    /// Add map controls (navigation, scale, etc.)
    pub fn add_map_controls(&mut self) -> Result<(), JsValue> {
        if let Some(map) = &self.map {
            self.control_manager.add_all_controls(map)
        } else {
            Err(JsValue::from_str("Map not initialized"))
        }
    }

    /// Set up map data sources and layers
    pub fn setup_map_data(
        &mut self,
        simulation_enabled: bool,
        tfl_data: TflDataRepository,
    ) -> Result<(), JsValue> {
        if let Some(map) = &self.map {
            // Register load event handler that will add layers
            self.event_manager
                .add_load_handler(map, move |map_instance| {
                    // When map loads, add the layers
                    layers::add_map_layers(&map_instance, simulation_enabled, tfl_data.clone())
                })
        } else {
            Err(JsValue::from_str("Map not initialized"))
        }
    }

    /// Update layer visibility based on TflLayers struct
    pub fn update_layer_visibility(&self, layers: &crate::app::TflLayers) -> Result<(), JsValue> {
        if let Some(map) = &self.map {
            self.layer_manager.update_visibility(map, layers)
        } else {
            Err(JsValue::from_str("Map not initialized"))
        }
    }
}

/// Implement Drop to clean up resources
impl Drop for MapLibreManager {
    fn drop(&mut self) {
        log::info_with_category(LogCategory::Map, "MapLibreManager being dropped");

        // Clear any global references
        if let Some(map) = &self.map {
            self.event_manager.clear_listeners(map);
            lifecycle::cleanup_map();
        }

        log::debug_with_category(LogCategory::Map, "Global references cleared");
    }
}
