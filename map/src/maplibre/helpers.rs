use crate::maplibre::bindings::*;
use crate::utils::geojson::{
    new_geojson_source, new_linestring_feature, new_point_feature, to_js_value,
};
use crate::utils::log::{LogCategory, with_context};
use js_sys::{Array, Object, Reflect};
use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::window;

// Helper to create a MapLibre map configuration
pub fn create_map_options(container_id: &str) -> Result<JsValue, JsValue> {
    with_context("create_map_options", LogCategory::Map, |logger| {
        logger.debug(&format!(
            "Creating map options for container: {}",
            container_id
        ));

        let options = Object::new();

        // Set required properties
        Reflect::set(
            &options,
            &JsValue::from_str("container"),
            &JsValue::from_str(container_id),
        )?;
        Reflect::set(
            &options,
            &JsValue::from_str("style"),
            &JsValue::from_str("https://tiles.openfreemap.org/styles/bright"),
        )?;

        // Set London center
        let center = Array::new();
        center.push(&JsValue::from_f64(-0.1275));
        center.push(&JsValue::from_f64(51.5072));
        Reflect::set(&options, &JsValue::from_str("center"), &center)?;

        // Set zoom level
        Reflect::set(
            &options,
            &JsValue::from_str("zoom"),
            &JsValue::from_f64(12.0),
        )?;

        // Set max bounds
        let bounds = Array::new();
        let sw = Array::new();
        sw.push(&JsValue::from_f64(-1.0)); // Westernmost (Reading)
        sw.push(&JsValue::from_f64(50.8)); // Southernmost (Coulsdon South)
        bounds.push(&sw);

        let ne = Array::new();
        ne.push(&JsValue::from_f64(0.7)); // Easternmost (Shenfield)
        ne.push(&JsValue::from_f64(52.6)); // Northernmost (Peterborough)
        bounds.push(&ne);

        Reflect::set(&options, &JsValue::from_str("maxBounds"), &bounds)?;

        logger.debug("Map options created successfully");
        Ok(options.into())
    })
}

// Create layer configuration
pub fn create_layer_groups() -> Result<JsValue, JsValue> {
    with_context("create_layer_groups", LogCategory::Map, |logger| {
        logger.debug("Creating layer groups");

        let layer_groups = Array::new();

        // Background group
        {
            // Label layer IDs - `window.mapInstance.getStyle().layers.slice(110, 119).map(layer => layer.id);`
            let label_ids = vec![
                "label_other",
                "label_village",
                "label_town",
                "label_state",
                "label_city",
                "label_city_capital",
                "label_country_3",
                "label_country_2",
                "label_country_1",
            ];

            // Create layers for each label ID and add to background_layers
            let label_layers = label_ids
                .iter()
                .map(|id| {
                    Layer::new(
                        &format!("labels-{}", &id[6..].replace('_', "-")), // "labels-other", ...
                        &format!("Map Label: {}", &id[6..].replace('_', " ")), // "Map Label: other", ...
                        id, // "label_other" - this is the name of the layer
                        false,
                    )
                })
                .collect::<Array>();

            let label_group = LayerGroup::new("Labels", &label_layers);
            layer_groups.push(&label_group);
        }

        // Label group
        {
            // Label layer IDs - `window.mapInstance.getStyle().layers.slice(110, 119).map(layer => layer.id);`
            let highway_ids = vec![
                "highway-name-path",
                "highway-name-minor",
                "highway-name-major",
                "highway-shield-non-us",
            ];

            // Create layers for each highway ID and add to highway_layers
            let highway_layers = highway_ids
                .iter()
                .map(|id| {
                    Layer::new(
                        &format!("highways-{}", &id[8..]), // "highways-name-path", ...
                        &format!("Highway: {}", &id[8..].replace('_', " ")), // "Highway: name path", ...
                        id, // "highway-name-path" - this is the name of the layer
                        false,
                    )
                })
                .collect::<Array>();

            let highway_group = LayerGroup::new("Highways", &highway_layers);
            layer_groups.push(&highway_group);
        }

        // Transport group
        {
            let transport_layers = Array::new();
            transport_layers.push(&Layer::new(
                "tube-central",
                "Central Line",
                "central-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "tube-northern",
                "Northern Line",
                "northern-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "tube-victoria",
                "Victoria Line",
                "victoria-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "tube-district",
                "District Line",
                "district-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "tube-bakerloo",
                "Bakerloo Line",
                "bakerloo-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "tube-hammersmith-city",
                "Hammersmith & City Line",
                "hammersmith-city-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "tube-piccadilly",
                "Piccadilly Line",
                "piccadilly-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "tube-jubilee",
                "Jubilee Line",
                "jubilee-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "tube-metropolitan",
                "Metropolitan Line",
                "metropolitan-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "tube-circle",
                "Circle Line",
                "circle-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "tube-waterloo-city",
                "Waterloo & City Line",
                "waterloo-city-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "elizabeth",
                "Elizabeth Line",
                "elizabeth-route-layer",
                true,
            ));
            // Non-tube
            transport_layers.push(&Layer::new(
                "overground-liberty",
                "Liberty Line",
                "liberty-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "overground-lioness",
                "Lioness Line",
                "lioness-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "overground-mildmay",
                "Mildmay Line",
                "mildmay-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "overground-suffragette",
                "Suffragette Line",
                "suffragette-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "overground-weaver",
                "Weaver Line",
                "weaver-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "overground-windrush",
                "Windrush Line",
                "windrush-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new(
                "cable-car",
                "Cable Car",
                "london-cable-car-route-layer",
                true,
            ));
            transport_layers.push(&Layer::new("dlr", "DLR", "dlr-route-layer", true));
            transport_layers.push(&Layer::new("tram", "Tram", "tram-route-layer", true));
            transport_layers.push(&Layer::new(
                "thameslink",
                "Thameslink",
                "thameslink-route-layer",
                true,
            ));

            let transport_group = LayerGroup::new("Transport", &transport_layers);
            layer_groups.push(&transport_group);
        }

        // Infrastructure group
        {
            let infrastructure_layers = Array::new();
            infrastructure_layers.push(&Layer::new(
                "stations",
                "Stations",
                "tfl-stations-layer",
                true,
            ));
            infrastructure_layers.push(&Layer::new(
                "station-labels",
                "Station Labels",
                "tfl-station-labels",
                true,
            ));

            let infrastructure_group = LayerGroup::new("Infrastructure", &infrastructure_layers);
            layer_groups.push(&infrastructure_group);
        }

        logger.debug("Layer groups created successfully");
        Ok(layer_groups.into())
    })
}

/// Helper to add scripts to the page programmatically
pub fn load_script(src: &str, on_load: Option<Closure<dyn FnMut()>>) -> Result<(), JsValue> {
    with_context("load_script", LogCategory::Map, |logger| {
        logger.debug(&format!("Loading script from '{}'", src));

        let window = window().ok_or_else(|| JsValue::from_str("No global window exists"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("No document exists on window"))?;

        let script = document.create_element("script")?;
        script.set_attribute("src", src)?;

        if let Some(callback) = on_load {
            logger.debug("Setting up onload callback");

            // Set onload property directly
            let callback_js_val = callback.as_ref().clone();
            Reflect::set(&script, &JsValue::from_str("onload"), &callback_js_val)?;

            // IMPORTANT: We must forget the closure to prevent it from being dropped
            logger.debug("Forgetting closure to keep it alive");
            callback.forget();
        }

        logger.debug("Appending script to document head");
        document
            .head()
            .ok_or_else(|| JsValue::from_str("document should have head"))?
            .append_child(&script)?;

        logger.info(&format!("Script '{}' added to document", src));
        Ok(())
    })
}

// Helper to add CSS to the page programmatically
pub fn load_css(href: &str) -> Result<(), JsValue> {
    with_context("load_css", LogCategory::Map, |logger| {
        logger.debug(&format!("Loading CSS from '{}'", href));

        let window = window().ok_or_else(|| JsValue::from_str("No global window exists"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("No document exists on window"))?;

        let link = document.create_element("link")?;
        link.set_attribute("rel", "stylesheet")?;
        link.set_attribute("href", href)?;

        document
            .head()
            .ok_or_else(|| JsValue::from_str("document should have head"))?
            .append_child(&link)?;

        logger.info(&format!("CSS '{}' loaded successfully", href));
        Ok(())
    })
}

// Helper to add inline script to the page
pub fn add_inline_script(content: &str) -> Result<(), JsValue> {
    with_context("add_inline_script", LogCategory::Map, |logger| {
        logger.debug("Adding inline script");

        let window = window().ok_or_else(|| JsValue::from_str("No global window exists"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("No document exists on window"))?;

        let script = document.create_element("script")?;
        script.set_inner_html(content);

        document
            .head()
            .ok_or_else(|| JsValue::from_str("document should have head"))?
            .append_child(&script)?;

        logger.debug("Inline script added successfully");
        Ok(())
    })
}

// Helper functions for creating map elements

pub fn create_line_layer(
    id: &str,
    source: &str,
    color: &str,
    width: f64,
) -> Result<JsValue, JsValue> {
    with_context("create_line_layer", LogCategory::Map, |logger| {
        logger.debug(&format!(
            "Creating line layer '{}' with source '{}'",
            id, source
        ));
        let layer = Object::new();
        Reflect::set(&layer, &JsValue::from_str("id"), &JsValue::from_str(id))?;
        Reflect::set(
            &layer,
            &JsValue::from_str("type"),
            &JsValue::from_str("line"),
        )?;
        Reflect::set(
            &layer,
            &JsValue::from_str("source"),
            &JsValue::from_str(source),
        )?;

        let layout = Object::new();
        Reflect::set(
            &layout,
            &JsValue::from_str("line-join"),
            &JsValue::from_str("round"),
        )?;
        Reflect::set(
            &layout,
            &JsValue::from_str("line-cap"),
            &JsValue::from_str("round"),
        )?;
        Reflect::set(&layer, &JsValue::from_str("layout"), &layout)?;

        let paint = Object::new();
        Reflect::set(
            &paint,
            &JsValue::from_str("line-color"),
            &JsValue::from_str(color),
        )?;
        Reflect::set(
            &paint,
            &JsValue::from_str("line-width"),
            &JsValue::from_f64(width),
        )?;
        Reflect::set(&layer, &JsValue::from_str("paint"), &paint)?;

        Ok(layer.into())
    })
}

/// Helper to create circle layer for stations
pub fn create_circle_layer(id: &str, source: &str) -> Result<JsValue, JsValue> {
    with_context("create_circle_layer", LogCategory::Map, |logger| {
        logger.debug(&format!(
            "Creating circle layer '{}' with source '{}'",
            id, source
        ));

        // Create the layer configuration with JSON
        let layer_config = serde_json::json!({
            "id": id,
            "type": "circle",
            "source": source,
            "paint": {
                "circle-radius": 6.0,
                "circle-color": "#ffffff",
                "circle-stroke-color": "#000000",
                "circle-stroke-width": 2.0
            }
        });

        // Serialize to JsValue
        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        match layer_config.serialize(&serializer) {
            Ok(js_value) => Ok(js_value),
            Err(err) => {
                logger.error(&format!("Failed to serialize circle layer: {:?}", err));
                Err(JsValue::from_str(&format!(
                    "Serialization error: {:?}",
                    err
                )))
            }
        }
    })
}

/// Helper to create a text label layer
pub fn create_label_layer(id: &str, source: &str) -> Result<JsValue, JsValue> {
    with_context("create_label_layer", LogCategory::Map, |logger| {
        logger.debug(&format!(
            "Creating label layer '{}' with source '{}'",
            id, source
        ));
        let layer = Object::new();

        Reflect::set(&layer, &JsValue::from_str("id"), &JsValue::from_str(id))?;
        Reflect::set(
            &layer,
            &JsValue::from_str("type"),
            &JsValue::from_str("symbol"),
        )?;
        Reflect::set(
            &layer,
            &JsValue::from_str("source"),
            &JsValue::from_str(source),
        )?;

        let layout = Object::new();
        let text_field = Array::new();
        text_field.push(&JsValue::from_str("get"));
        text_field.push(&JsValue::from_str("name"));

        Reflect::set(&layout, &JsValue::from_str("text-field"), &text_field)?;

        let font = Array::new();
        font.push(&JsValue::from_str("Noto Sans Regular"));
        Reflect::set(&layout, &JsValue::from_str("text-font"), &font)?;

        let offset = Array::new();
        offset.push(&JsValue::from_f64(0.0));
        offset.push(&JsValue::from_f64(1.5));
        Reflect::set(&layout, &JsValue::from_str("text-offset"), &offset)?;

        Reflect::set(
            &layout,
            &JsValue::from_str("text-anchor"),
            &JsValue::from_str("top"),
        )?;
        Reflect::set(&layer, &JsValue::from_str("layout"), &layout)?;

        let paint = Object::new();
        Reflect::set(
            &paint,
            &JsValue::from_str("text-color"),
            &JsValue::from_str("#000000"),
        )?;
        Reflect::set(
            &paint,
            &JsValue::from_str("text-halo-color"),
            &JsValue::from_str("#ffffff"),
        )?;
        Reflect::set(
            &paint,
            &JsValue::from_str("text-halo-width"),
            &JsValue::from_f64(2.0),
        )?;
        Reflect::set(&layer, &JsValue::from_str("paint"), &paint)?;

        Ok(layer.into())
    })
}

pub fn create_scale_control_options() -> Result<JsValue, JsValue> {
    with_context("create_scale_control_options", LogCategory::Map, |logger| {
        logger.debug("Creating scale control options");
        let options = Object::new();
        Reflect::set(
            &options,
            &JsValue::from_str("maxWidth"),
            &JsValue::from_f64(100.0),
        )?;
        Reflect::set(
            &options,
            &JsValue::from_str("unit"),
            &JsValue::from_str("metric"),
        )?;
        Ok(options.into())
    })
}
