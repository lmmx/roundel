use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// From the TfL topology data model, as recorded here:
// https://github.com/lmmx/tubeulator/blob/a8fc10becac3ea04cf16b91b0c24be944df692a5/src/tubeulator/topology/data_model.py

/// Represents a TfL station with its location and metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Station {
    /// Unique identifier for the station
    #[serde(rename = "StationUniqueId")]
    pub station_unique_id: String,
    /// Human-readable name of the station
    #[serde(rename = "StationName")]
    pub station_name: String,
    /// Fare zones the station belongs to (comma-separated)
    #[serde(rename = "FareZones")]
    pub fare_zones: String,
    /// Optional hub Naptan code for interchanges
    #[serde(rename = "HubNaptanCode")]
    #[serde(default)]
    pub hub_naptan_code: Option<String>,
    /// Whether the station has Wi-Fi
    #[serde(rename = "Wifi")]
    #[serde(default)]
    pub wifi: bool,
    /// Unique ID for outside of the station
    #[serde(rename = "OutsideStationUniqueId")]
    pub outside_station_unique_id: String,
    /// Latitude coordinate of the station
    #[serde(rename = "Lat")]
    pub lat: f64,
    /// Longitude coordinate of the station
    #[serde(rename = "Lon")]
    pub lon: f64,
    /// List of component station codes that make up this station
    #[serde(rename = "ComponentStations")]
    #[serde(default)]
    pub component_stations: Vec<String>,
}

/// Represents a platform at a TfL station
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Platform {
    /// Unique identifier for this platform
    #[serde(rename = "PlatformUniqueId")]
    pub platform_unique_id: String,
    /// Station this platform belongs to
    #[serde(rename = "StationUniqueId")]
    pub station_unique_id: String,
    /// Platform number (as string to handle complex numbering)
    #[serde(rename = "PlatformNumber")]
    #[serde(default)]
    pub platform_number: Option<String>,
    /// Direction of travel (Northbound, Southbound, etc.)
    #[serde(rename = "CardinalDirection")]
    #[serde(default)]
    pub cardinal_direction: Option<String>,
    /// Optional platform Naptan code
    #[serde(rename = "PlatformNaptanCode")]
    #[serde(default)]
    pub platform_naptan_code: Option<String>,
    /// Human-readable name for the platform
    #[serde(rename = "PlatformFriendlyName")]
    pub platform_friendly_name: String,
    /// Whether the platform is accessible to customers
    #[serde(rename = "IsCustomerFacing")]
    pub is_customer_facing: bool,
    /// Whether the platform has service interchange
    #[serde(rename = "HasServiceInterchange")]
    pub has_service_interchange: bool,
    /// Name of the station this platform is in
    #[serde(rename = "StationName")]
    pub station_name: String,
    /// Fare zones for this station
    #[serde(rename = "FareZones")]
    pub fare_zones: String,
    /// Hub Naptan code if applicable
    #[serde(rename = "HubNaptanCode")]
    #[serde(default)]
    pub hub_naptan_code: Option<String>,
    /// Whether the station has Wi-Fi
    #[serde(rename = "Wifi")]
    #[serde(default)]
    pub wifi: bool,
    /// Outside station unique ID
    #[serde(rename = "OutsideStationUniqueId")]
    pub outside_station_unique_id: String,
    /// Stop area Naptan code
    #[serde(rename = "StopAreaNaptanCode")]
    pub stop_area_naptan_code: String,
    /// Line this platform serves (e.g., "central", "district")
    #[serde(rename = "Line")]
    pub line: String,
    /// Direction this platform heads toward
    #[serde(rename = "DirectionTowards")]
    #[serde(default)]
    pub direction_towards: Option<String>,
    /// Platform service group name if applicable
    #[serde(rename = "PlatformServiceGroupName")]
    #[serde(default)]
    pub platform_service_group_name: Option<String>,
}

/// Response structure from the stations API
#[derive(Debug, Deserialize)]
pub struct StationsResponse {
    // pub context: ResponseContext,
    pub success: bool,
    pub results: Vec<Station>,
}

/// Response structure from the platforms API
#[derive(Debug, Deserialize)]
pub struct PlatformsResponse {
    // pub context: ResponseContext,
    pub success: bool,
    pub results: Vec<Platform>,
}

// /// Context information included in API responses
// #[derive(Debug, Deserialize)]
// pub struct ResponseContext {
//     pub request_time: String,
//     pub response_time: String,
//     pub response_latency: f64,
//     pub query: String,
// }

/// Represents a TfL route sequence with line information and stations
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RouteSequence {
    /// Unique identifier for the line
    #[serde(rename = "LineId")]
    pub line_id: String,
    // /// Human-readable name of the line
    // #[serde(rename = "LineName")]
    // pub line_name: String,
    /// Direction of the route (inbound/outbound)
    #[serde(rename = "Direction")]
    pub direction: String,
    // /// Whether this is an outbound-only route
    // #[serde(rename = "IsOutboundOnly")]
    // pub is_outbound_only: bool,
    /// Transport mode (tube, bus, etc.)
    #[serde(rename = "Mode")]
    pub mode: String,
    /// GeoJSON LineString representation of the route
    #[serde(rename = "LineStrings")]
    pub line_strings: Vec<String>,
    // /// Stations along this route
    // #[serde(rename = "Stations")]
    // pub stations: Vec<MatchedStop>,
    // /// Detailed stop point sequences
    // #[serde(rename = "StopPointSequences")]
    // #[serde(default)]
    // pub stop_point_sequences: Vec<StopPointSequence>,
    // /// Ordered line routes
    // #[serde(rename = "OrderedLineRoutes")]
    // #[serde(default)]
    // pub ordered_line_routes: Vec<OrderedRoute>,
}

/// Represents a stop point sequence for a line
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StopPointSequence {
    /// Unique identifier for the line
    #[serde(rename = "LineId")]
    pub line_id: String,
    /// Human-readable name of the line
    #[serde(rename = "LineName")]
    pub line_name: String,
    /// Direction of the sequence
    #[serde(rename = "Direction")]
    pub direction: String,
    /// Branch identifier
    #[serde(rename = "BranchId")]
    pub branch_id: i32,
    /// Next branch identifiers
    #[serde(rename = "NextBranchIds")]
    #[serde(default)]
    pub next_branch_ids: Vec<i32>,
    /// Previous branch identifiers
    #[serde(rename = "PrevBranchIds")]
    #[serde(default)]
    pub prev_branch_ids: Vec<i32>,
    /// Stop points in this sequence
    #[serde(rename = "StopPoint")]
    #[serde(default)]
    pub stop_point: Vec<MatchedStop>,
    /// Service type
    #[serde(rename = "ServiceType")]
    #[serde(default)]
    pub service_type: Option<String>,
}

/// Represents a station stop on a route
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MatchedStop {
    /// Route identifier
    #[serde(rename = "RouteId")]
    #[serde(default)]
    pub route_id: Option<i32>,
    /// Parent station identifier
    #[serde(rename = "ParentId")]
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Station identifier
    #[serde(rename = "StationId")]
    #[serde(default)]
    pub station_id: Option<String>,
    /// ICS identifier
    #[serde(rename = "IcsId")]
    #[serde(default)]
    pub ics_id: Option<String>,
    /// Top-most parent identifier
    #[serde(rename = "TopMostParentId")]
    #[serde(default)]
    pub top_most_parent_id: Option<String>,
    /// Direction
    #[serde(rename = "Direction")]
    #[serde(default)]
    pub direction: Option<String>,
    /// Towards destination
    #[serde(rename = "Towards")]
    #[serde(default)]
    pub towards: Option<String>,
    /// Transport modes
    #[serde(rename = "Modes")]
    #[serde(default)]
    pub modes: Vec<String>,
    /// Stop type
    #[serde(rename = "StopType")]
    #[serde(default)]
    pub stop_type: Option<String>,
    /// Stop letter
    #[serde(rename = "StopLetter")]
    #[serde(default)]
    pub stop_letter: Option<String>,
    /// Fare zone
    #[serde(rename = "Zone")]
    #[serde(default)]
    pub zone: Option<String>,
    /// Accessibility summary
    #[serde(rename = "AccessibilitySummary")]
    #[serde(default)]
    pub accessibility_summary: Option<String>,
    /// Whether disruption is occurring
    #[serde(rename = "HasDisruption")]
    #[serde(default)]
    pub has_disruption: Option<bool>,
    /// Lines serving this stop
    #[serde(rename = "Lines")]
    #[serde(default)]
    pub lines: Vec<LineIdentifier>,
    /// Status
    #[serde(rename = "Status")]
    #[serde(default)]
    pub status: Option<bool>,
    /// Unique identifier
    #[serde(rename = "Id")]
    #[serde(default)]
    pub id: Option<String>,
    /// URL
    #[serde(rename = "Url")]
    #[serde(default)]
    pub url: Option<String>,
    /// Name of the stop
    #[serde(rename = "Name")]
    #[serde(default)]
    pub name: Option<String>,
    /// Latitude
    #[serde(rename = "Lat")]
    #[serde(default)]
    pub lat: Option<f64>,
    /// Longitude
    #[serde(rename = "Lon")]
    #[serde(default)]
    pub lon: Option<f64>,
}

/// Represents a line identifier
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LineIdentifier {
    /// Unique identifier
    #[serde(rename = "Id")]
    pub id: String,
    /// Name of the line
    #[serde(rename = "Name")]
    pub name: String,
    /// URI
    #[serde(rename = "Uri")]
    #[serde(default)]
    pub uri: Option<String>,
    /// Full name
    #[serde(rename = "FullName")]
    #[serde(default)]
    pub full_name: Option<String>,
    /// Type
    #[serde(rename = "Type")]
    #[serde(default)]
    pub type_name: Option<String>,
    /// Crowding information (simplified to avoid complex nesting)
    #[serde(rename = "Crowding")]
    #[serde(default)]
    pub crowding: Option<Crowding>,
    /// Route type
    #[serde(rename = "RouteType")]
    #[serde(default)]
    pub route_type: Option<String>,
    /// Status
    #[serde(rename = "Status")]
    #[serde(default)]
    pub status: Option<String>,
}

/// Simplified crowding information structure
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Crowding {
    /// Passenger flows (always empty in responses)
    #[serde(rename = "PassengerFlows")]
    #[serde(default)]
    pub passenger_flows: Vec<serde_json::Value>,
    /// Train loadings (always empty in responses)
    #[serde(rename = "TrainLoadings")]
    #[serde(default)]
    pub train_loadings: Vec<serde_json::Value>,
}

/// Represents an ordered route
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderedRoute {
    /// Name of the route
    #[serde(rename = "Name")]
    pub name: String,
    /// Naptan IDs for stops on this route
    #[serde(rename = "NaptanIds")]
    #[serde(default)]
    pub naptan_ids: Vec<String>,
    /// Service type
    #[serde(rename = "ServiceType")]
    #[serde(default)]
    pub service_type: Option<String>,
}

/// Response structure for a single route
#[derive(Debug, Deserialize)]
pub struct RouteResponse {
    // pub context: ResponseContext,
    pub success: bool,
    pub results: Vec<RouteSequence>,
}

/// Structure for the combined routes file
#[derive(Debug, Deserialize)]
pub struct RoutesFile {
    #[serde(flatten)]
    pub routes: HashMap<String, HashMap<String, RouteResponse>>,
}
