// Layer management for map
use crate::data::TflDataRepository;
use crate::maplibre::bindings::Map;
use crate::maplibre::helpers::{create_circle_layer, create_label_layer, create_line_layer};
use crate::utils::log::{LogCategory, with_context};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Manager for map layers
pub struct LayerManager {
    registered_layers: HashMap<String, LayerInfo>,
}

/// Information about a registered layer
struct LayerInfo {
    id: String,
    type_: String,
    source: String,
    visible: bool,
}

impl LayerManager {
    pub fn new() -> Self {
        Self {
            registered_layers: HashMap::new(),
        }
    }

    /// Add a layer to the map
    pub fn add_layer(
        &mut self,
        map: &Map,
        layer_id: &str,
        source: &str,
        type_: &str,
        style: impl Into<JsValue>,
    ) -> Result<(), JsValue> {
        with_context("LayerManager::add_layer", LogCategory::Map, |logger| {
            logger.debug(&format!(
                "Adding layer '{}' with source '{}'",
                layer_id, source
            ));

            // Add layer to map
            let layer = match type_ {
                "line" => create_line_layer(layer_id, source, "#FFFFFF", 1.0)?, // Default values
                "circle" => create_circle_layer(layer_id, source)?,
                "label" => create_label_layer(layer_id, source)?,
                _ => return Err(JsValue::from_str("Unsupported layer type")),
            };

            // Apply additional style if provided
            if !style.into().is_undefined() {
                // TODO: Apply additional style properties
            }

            map.add_layer(&layer);

            // Register the layer
            self.registered_layers.insert(
                layer_id.to_string(),
                LayerInfo {
                    id: layer_id.to_string(),
                    type_: type_.to_string(),
                    source: source.to_string(),
                    visible: true, // Default to visible
                },
            );

            logger.debug(&format!("Layer '{}' added and registered", layer_id));

            Ok(())
        })
    }

    /// Set the visibility of a layer
    pub fn set_visibility(
        &mut self,
        map: &Map,
        layer_id: &str,
        visible: bool,
    ) -> Result<(), JsValue> {
        with_context("LayerManager::set_visibility", LogCategory::Map, |logger| {
            if map.get_layer(layer_id).is_some() {
                logger.debug(&format!(
                    "Setting '{}' visibility to {}",
                    layer_id,
                    if visible { "visible" } else { "none" }
                ));

                let visibility = if visible { "visible" } else { "none" };
                map.set_layout_property(layer_id, "visibility", &JsValue::from_str(visibility));

                // Update our tracking information
                if let Some(layer_info) = self.registered_layers.get_mut(layer_id) {
                    layer_info.visible = visible;
                }

                Ok(())
            } else {
                logger.error(&format!("Layer '{}' not found", layer_id));
                Err(JsValue::from_str(&format!(
                    "Layer '{}' not found",
                    layer_id
                )))
            }
        })
    }

    /// Update layer visibility based on TflLayers struct
    pub fn update_visibility(
        &self,
        map: &Map,
        layers: &crate::app::TflLayers,
    ) -> Result<(), JsValue> {
        with_context(
            "LayerManager::update_visibility",
            LogCategory::Map,
            |logger| {
                // Helper function to set visibility
                let set_visibility = |layer_id: &str, visible: bool| -> Result<(), JsValue> {
                    logger.debug(&format!("Checking if layer '{}' exists", layer_id));
                    if map.get_layer(layer_id).is_some() {
                        logger.debug(&format!(
                            "Setting '{}' visibility to {}",
                            layer_id,
                            if visible { "visible" } else { "none" }
                        ));
                        let visibility = if visible { "visible" } else { "none" };
                        map.set_layout_property(
                            layer_id,
                            "visibility",
                            &JsValue::from_str(visibility),
                        );
                    } else {
                        logger.debug(&format!("Layer '{}' not found, skipping", layer_id));
                    }
                    Ok(())
                };

                // Update tube layers
                set_visibility("london-cable-car-line-layer", layers.tube)?;

                // Update stations layers
                set_visibility("stations-layer", layers.stations)?;
                set_visibility("station-labels", layers.stations)?;

                Ok(())
            },
        )
    }
}

/// Helper function to add MapLibre layers
pub fn add_map_layers(
    map_instance: &JsValue,
    simulation_enabled: bool,
    tfl_data: TflDataRepository,
) -> Result<(), JsValue> {
    with_context("add_map_layers", LogCategory::Map, |logger| {
        logger.debug("Creating map layers");

        let map: Map = map_instance.clone().into();
        logger.debug("Map instance cloned");

        if tfl_data.is_loaded {
            // Add all stations as a GeoJSON source
            logger.info("Adding all TfL stations to the map");

            // Convert stations to GeoJSON
            let stations_geojson = crate::data::stations_to_geojson(&tfl_data.stations)?;
            map.add_source("tfl-stations", &stations_geojson);
            logger.debug("TfL stations source added");

            // Add a circle layer for the stations
            let stations_layer = create_circle_layer("tfl-stations-layer", "tfl-stations")?;
            map.add_layer(&stations_layer);
            logger.debug("TfL stations layer added");

            // Add a label layer for the stations
            let labels_layer = create_label_layer("tfl-station-labels", "tfl-stations")?;
            map.add_layer(&labels_layer);
            logger.debug("TfL station labels layer added");

            // Add all tube lines
            logger.info("Adding TfL lines to the map");

            // match crate::data::generate_all_line_data(&tfl_data) {
            //     Ok(line_data) => {
            //         let line_count = line_data.len(); // Store the length before moving line_data

            //         for (line_name, line_geojson, color) in line_data {
            //             let source_id = format!("{}-line", line_name);
            //             let layer_id = format!("{}-line-layer", line_name);

            //             // Add the source
            //             map.add_source(&source_id, &line_geojson);

            //             // Add the layer
            //             let line_layer = create_line_layer(&layer_id, &source_id, &color, 4.0)?;
            //             map.add_layer(&line_layer);

            //             logger.debug(&format!("Added {} line", line_name));
            //         }

            //         logger.info(&format!("Added {} TfL lines to the map", line_count));
            //     }
            //     Err(e) => {
            //         logger.error(&format!("Failed to generate line data: {:?}", e));
            //     }
            // }
        }

        logger.info("All map layers added successfully");

        if simulation_enabled {
            // Initialize the vehicle simulation after all other layers are added
            logger.info("Initializing vehicle simulation from map_layers");
            let init_simulation_js = r#"
                if (typeof window.initializeSimulation === 'function') {
                    console.log('Calling window.initializeSimulation()');
                    window.initializeSimulation();
                } else {
                    console.log('Creating initializeSimulation placeholder');
                    // Create a placeholder function that will be replaced when the simulation module loads
                    window.initializeSimulation = function() {
                        console.log('Placeholder initializeSimulation called - will retry in 1 second');
                        setTimeout(() => {
                            if (typeof window.realInitializeSimulation === 'function') {
                                window.realInitializeSimulation();
                            }
                        }, 1000);
                    };
                }
            "#;
            let _ = js_sys::eval(init_simulation_js);
            logger.debug("Vehicle simulation initialization requested");
        } else {
            logger.debug("Simulation disabled, skipping initialization");
        }

        Ok(())
    })
}
