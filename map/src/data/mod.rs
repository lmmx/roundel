pub mod api;
pub mod line_definitions;
pub mod loader;
pub mod map_helpers;
pub mod model;

// Re-export commonly used items
pub use map_helpers::{generate_all_route_geometries, stations_to_geojson};

use crate::utils::log::{self, LogCategory};
use std::collections::HashMap;

/// A consolidated data repository for TfL data
#[derive(Clone, Default)]
pub struct TflDataRepository {
    /// All stations with valid coordinates
    pub stations: Vec<model::Station>,
    /// All platforms grouped by station ID
    pub platforms_by_station: HashMap<String, Vec<model::Platform>>,
    /// Stations by their unique ID for quick lookup
    pub station_by_id: HashMap<String, model::Station>,
    /// Route data organized by line ID and direction
    pub routes: HashMap<String, HashMap<String, Vec<model::RouteSequence>>>,
    /// Route geometries by line ID for efficient rendering
    pub route_geometries: HashMap<String, Vec<Vec<[f64; 2]>>>,
    /// Indicates if the repository has been loaded
    pub is_loaded: bool,
}

impl TflDataRepository {
    /// Initialize the data repository by loading all data
    pub async fn initialize(load_buses: bool) -> Result<Self, String> {
        log::info_with_category(LogCategory::App, "Initializing TFL data repository");

        // Load and process stations
        let stations = loader::load_stations().await?;
        let valid_stations = loader::filter_valid_stations(stations);

        // Create lookup map for stations
        let station_by_id = valid_stations
            .iter()
            .map(|s| (s.station_unique_id.clone(), s.clone()))
            .collect();

        // Load and process platforms
        let platforms = loader::load_platforms().await?;
        let platforms_by_station = loader::group_platforms_by_station(platforms);

        // Load and process routes
        let routes = loader::load_routes(load_buses).await?;
        let route_geometries = loader::process_route_geometries(&routes);

        log::info_with_category(
            LogCategory::App,
            &format!(
                "TFL data repository initialized with {} stations and {} routes",
                valid_stations.len(),
                routes.len(),
            ),
        );

        Ok(Self {
            stations: valid_stations,
            platforms_by_station,
            station_by_id,
            routes,
            route_geometries,
            is_loaded: true,
        })
    }

    // /// Get a station by its unique ID
    // pub fn get_station(&self, station_id: &str) -> Option<&model::Station> {
    //     self.station_by_id.get(station_id)
    // }

    // /// Get platforms for a specific station
    // pub fn get_platforms_for_station(&self, station_id: &str) -> Vec<&model::Platform> {
    //     match self.platforms_by_station.get(station_id) {
    //         Some(platforms) => platforms.iter().collect(),
    //         None => Vec::new(),
    //     }
    // }

    // /// Get all stations for a specific line
    // pub fn get_stations_for_line(&self, line_name: &str) -> Vec<&model::Station> {
    //     let mut result = Vec::new();

    //     // Check each station's platforms to see if any serve this line
    //     for (station_id, platforms) in &self.platforms_by_station {
    //         let serves_line = platforms.iter().any(|p| p.line == line_name);

    //         if serves_line {
    //             if let Some(station) = self.station_by_id.get(station_id) {
    //                 result.push(station);
    //             }
    //         }
    //     }

    //     result
    // }

    // /// Get all route sequences for a specific line
    // pub fn get_routes_for_line(&self, line_id: &str) -> Vec<&model::RouteSequence> {
    //     let mut result = Vec::new();

    //     if let Some(directions) = self.routes.get(line_id) {
    //         for (_, sequences) in directions {
    //             for sequence in sequences {
    //                 result.push(sequence);
    //             }
    //         }
    //     }

    //     result
    // }

    // /// Get route geometries for a specific line
    // pub fn get_route_geometries_for_line(&self, line_id: &str) -> Vec<Vec<[f64; 2]>> {
    //     match self.route_geometries.get(line_id) {
    //         Some(geometries) => geometries.clone(),
    //         None => Vec::new(),
    //     }
    // }
}
