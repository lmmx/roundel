// src/utils/geojson.rs
use serde::Serialize;
use wasm_bindgen::{JsError, JsValue};

/// GeoJSON source specification
#[derive(Debug, Serialize)]
pub struct GeoJsonSource {
    #[serde(rename = "type")]
    pub source_type: &'static str, // Always "geojson"
    pub data: FeatureCollection,
}

/// GeoJSON FeatureCollection
#[derive(Debug, Serialize)]
pub struct FeatureCollection {
    #[serde(rename = "type")]
    pub collection_type: &'static str, // Always "FeatureCollection"
    pub features: Vec<Feature>,
}

/// GeoJSON Feature
#[derive(Debug, Serialize)]
pub struct Feature {
    #[serde(rename = "type")]
    pub feature_type: &'static str, // Always "Feature"
    pub geometry: Geometry,
    pub properties: serde_json::Value,
}

/// GeoJSON Geometry (union type)
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Geometry {
    #[serde(rename = "Point")]
    Point { coordinates: [f64; 2] },

    #[serde(rename = "LineString")]
    LineString { coordinates: Vec<[f64; 2]> },
    // Add other geometry types as needed
}

/// Create a new GeoJSON source with a FeatureCollection
pub fn new_geojson_source(features: Vec<Feature>) -> GeoJsonSource {
    GeoJsonSource {
        source_type: "geojson",
        data: FeatureCollection {
            collection_type: "FeatureCollection",
            features,
        },
    }
}

/// Create a new point feature
pub fn new_point_feature(lon: f64, lat: f64, properties: serde_json::Value) -> Feature {
    Feature {
        feature_type: "Feature",
        geometry: Geometry::Point {
            coordinates: [lon, lat],
        },
        properties,
    }
}

/// Create a new line string feature
pub fn new_linestring_feature(
    coordinates: Vec<[f64; 2]>,
    properties: serde_json::Value,
) -> Feature {
    Feature {
        feature_type: "Feature",
        geometry: Geometry::LineString { coordinates },
        properties,
    }
}

/// Serialize GeoJSON to JsValue
pub fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);

    match value.serialize(&serializer) {
        Ok(js_value) => Ok(js_value),
        Err(err) => Err(JsError::new(&format!(
            "Failed to serialize GeoJSON: {:?}",
            err
        ))),
    }
}
