use super::model::{Platform, Station};
use crate::data::TflDataRepository;
use crate::utils::geojson::{
    new_geojson_source, new_linestring_feature, new_point_feature, to_js_value,
};
use crate::utils::log::{self, LogCategory};
use std::collections::HashMap;
use wasm_bindgen::{JsError, JsValue};

/// Convert a list of stations into a format suitable for MapLibre GeoJSON
pub fn stations_to_geojson(stations: &[Station]) -> Result<JsValue, JsError> {
    log::info_with_category(
        LogCategory::Map,
        &format!("Converting {} stations to GeoJSON", stations.len()),
    );

    let features: Vec<_> = stations
        .iter()
        .filter(|station| !station.lat.is_nan() && !station.lon.is_nan())
        .map(|station| {
            // Create properties
            let properties = serde_json::json!({
                "id": station.station_unique_id,
                "name": station.station_name,
                "fareZones": station.fare_zones,
                "wifi": station.wifi,
            });

            // Create the feature using our helper
            new_point_feature(station.lon, station.lat, properties)
        })
        .collect();

    // Log the count
    let feature_count = features.len();
    log::debug_with_category(
        LogCategory::Map,
        &format!("Created GeoJSON with {} features", feature_count),
    );

    // Create the source and serialise
    let geojson_source = new_geojson_source(features);
    to_js_value(&geojson_source)
}

/// Create a mapping of line names to their corresponding stations
pub fn create_line_stations_map(platforms: &[Platform]) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();

    for platform in platforms {
        // Skip non-TfL lines or special cases
        if platform.line == "national-rail" || platform.line.is_empty() {
            continue;
        }

        map.entry(platform.line.clone())
            .or_insert_with(Vec::new)
            .push(platform.station_unique_id.clone());
    }

    // Deduplicate station IDs for each line
    for stations in map.values_mut() {
        stations.sort();
        stations.dedup();
    }

    map
}

/// Convert line stations to GeoJSON LineString format
/// Convert line stations to GeoJSON LineString format
pub fn line_to_geojson(
    line_name: &str,
    station_ids: &[String],
    stations_by_id: &HashMap<String, Station>,
) -> Result<JsValue, JsError> {
    log::info_with_category(
        LogCategory::Map,
        &format!(
            "Creating GeoJSON for {} line with {} stations",
            line_name,
            station_ids.len()
        ),
    );

    // Get coordinates for all stations on this line
    let coordinates: Vec<[f64; 2]> = station_ids
        .iter()
        .filter_map(|id| stations_by_id.get(id))
        .map(|station| [station.lon, station.lat])
        .collect();

    // We need at least 2 points to form a line
    if coordinates.len() < 2 {
        return Err(JsError::new(&format!(
            "Not enough stations with valid coordinates for {} line",
            line_name
        )));
    }

    // Create properties and feature
    let properties = serde_json::json!({ "name": line_name });
    let feature = new_linestring_feature(coordinates.clone(), properties);

    // Create source and log
    let geojson_source = new_geojson_source(vec![feature]);
    log::debug_with_category(
        LogCategory::Map,
        &format!(
            "Created GeoJSON LineString for {} line with {} points",
            line_name,
            coordinates.len()
        ),
    );

    to_js_value(&geojson_source)
}

/// Get the color for a specific TfL line
pub fn get_line_color(line_name: &str) -> &'static str {
    match line_name {
        "bakerloo" => "#B36305",
        "central" => "#E32017",
        "circle" => "#FFD300",
        "district" => "#00782A",
        "dlr" => "#00A4A7",
        "elizabeth" => "#6950A1",
        "hammersmith-city" => "#F3A9BB",
        "jubilee" => "#A0A5A9",
        "london-cable-car" => "#AF174C",
        "london-overground" => "#EE7C0E",
        "metropolitan" => "#9B0056",
        "northern" => "#000000",
        "piccadilly" => "#003688",
        "thameslink" => "#C1007C",
        "tram" => "#84B817",
        "victoria" => "#0098D4",
        "waterloo-city" => "#95CDBA",
        "liberty" => "#4C6366",
        "lioness" => "#FFA32B",
        "mildmay" => "#088ECC",
        "suffragette" => "#59C274",
        "weaver" => "#B43983",
        "windrush" => "#FF2E24",
        _ => "#FFFFFF", // Default white for unknown lines
    }
}

// Not used: left in for debugging (if there's a new line without routes, uncomment use in app/mod.rs)
/// Generate all line data for MapLibre
#[allow(dead_code)]
pub fn generate_all_line_data(
    repository: &super::TflDataRepository,
) -> Result<Vec<(String, JsValue, String)>, JsValue> {
    log::info_with_category(LogCategory::Map, "Generating data for all TfL lines");

    // Collect all platforms from repository into a single Vec<Platform>
    let platforms: Vec<Platform> = repository
        .platforms_by_station
        .values()
        .flat_map(|v| v.clone())
        .collect();

    // Create map of line names to station IDs
    let line_stations = create_line_stations_map(&platforms);

    // Generate GeoJSON for each line
    let mut result = Vec::new();

    for (line_name, station_ids) in line_stations {
        // Skip lines with too few stations
        if station_ids.len() < 2 {
            log::debug_with_category(
                LogCategory::Map,
                &format!(
                    "Skipping {} line with only {} stations",
                    line_name,
                    station_ids.len()
                ),
            );
            continue;
        }

        match line_to_geojson(&line_name, &station_ids, &repository.station_by_id) {
            Ok(geojson) => {
                let color = get_line_color(&line_name);
                result.push((line_name, geojson, color.to_string()));
            }
            Err(e) => {
                log::error_with_category(
                    LogCategory::Map,
                    &format!("Failed to generate GeoJSON for {} line: {:?}", line_name, e),
                );
            }
        }
    }

    log::info_with_category(
        LogCategory::Map,
        &format!("Generated data for {} TfL lines", result.len()),
    );

    Ok(result)
}

/// Convert route geometries for a specific line to GeoJSON
pub fn route_geometries_to_geojson(
    line_id: &str,
    geometries: &Vec<Vec<[f64; 2]>>,
) -> Result<JsValue, JsError> {
    // Create features for each non-empty geometry
    let features: Vec<_> = geometries
        .iter()
        .enumerate()
        .filter(|(_, coords)| !coords.is_empty())
        .map(|(i, coords)| {
            let properties = serde_json::json!({
                "line_id": line_id,
                "segment_id": i,
            });

            new_linestring_feature(coords.clone(), properties)
        })
        .collect();

    // Create the source and serialize
    let geojson_source = new_geojson_source(features);
    to_js_value(&geojson_source)
}

/// Generate all route geometries as GeoJSON for multiple lines
pub fn generate_all_route_geometries(
    tfl_data: &TflDataRepository,
) -> Result<Vec<(String, JsValue)>, JsError> {
    let mut result = Vec::new();

    // Process each line
    for (line_id, geometries) in &tfl_data.route_geometries {
        // Skip lines with no geometries
        if geometries.is_empty() {
            continue;
        }

        match route_geometries_to_geojson(line_id, geometries) {
            Ok(geojson) => {
                result.push((line_id.clone(), geojson));
            }
            Err(err) => {
                log::error_with_category(
                    LogCategory::Map,
                    &format!("Failed to convert route to GeoJSON: {:?}", err),
                );
            }
        }
    }

    Ok(result)
}
