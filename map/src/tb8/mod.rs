//! # tb8-rs Module
//!
//! Models for the TfL API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Define core models equivalent to the Python Pydantic models

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaData {
    pub request_time: DateTime<Utc>,
    pub response_time: DateTime<Utc>,
    pub response_latency: f64,  // Duration in seconds
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub context: MetaData,
    pub success: bool,
    pub results: Vec<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub context: MetaData,
    pub success: bool,
    pub error: String,
}

// Line and Station models

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Line {
    pub id: String,
    pub name: String,
    #[serde(rename = "modeName")]
    pub mode_name: String,
    #[serde(default)]
    pub disruptions: Vec<Disruption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<DateTime<Utc>>,
    #[serde(default)]
    #[serde(rename = "lineStatuses")]
    pub line_statuses: Vec<LineStatus>,
    #[serde(default)]
    #[serde(rename = "routeSections")]
    pub route_sections: Vec<RouteSection>,
    #[serde(default)]
    #[serde(rename = "serviceTypes")]
    pub service_types: Vec<LineServiceType>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LineStatus {
    pub id: Option<i32>,
    #[serde(rename = "lineId")]
    pub line_id: Option<String>,
    #[serde(rename = "statusSeverity")]
    pub status_severity: Option<i32>,
    #[serde(rename = "statusSeverityDescription")]
    pub status_severity_description: Option<String>,
    pub reason: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
    #[serde(default)]
    #[serde(rename = "validityPeriods")]
    pub validity_periods: Vec<ValidityPeriod>,
    pub disruption: Option<Disruption>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidityPeriod {
    #[serde(rename = "fromDate")]
    pub from_date: Option<DateTime<Utc>>,
    #[serde(rename = "toDate")]
    pub to_date: Option<DateTime<Utc>>,
    #[serde(rename = "isNow")]
    pub is_now: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Disruption {
    pub category: Option<String>,
    #[serde(rename = "type")]
    pub disruption_type: Option<String>,
    #[serde(rename = "categoryDescription")]
    pub category_description: Option<String>,
    pub description: Option<String>,
    pub summary: Option<String>,
    #[serde(rename = "additionalInfo")]
    pub additional_info: Option<String>,
    pub created: Option<DateTime<Utc>>,
    #[serde(rename = "lastUpdate")]
    pub last_update: Option<DateTime<Utc>>,
    #[serde(default)]
    #[serde(rename = "affectedRoutes")]
    pub affected_routes: Vec<RouteSection>,
    #[serde(default)]
    #[serde(rename = "affectedStops")]
    pub affected_stops: Vec<StopPoint>,
    #[serde(rename = "closureText")]
    pub closure_text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RouteSection {
    pub id: Option<String>,
    #[serde(rename = "lineId")]
    pub line_id: Option<String>,
    #[serde(rename = "routeCode")]
    pub route_code: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "lineString")]
    pub line_string: Option<String>,
    pub direction: Option<String>,
    #[serde(rename = "originationName")]
    pub origination_name: Option<String>,
    #[serde(rename = "destinationName")]
    pub destination_name: Option<String>,
    #[serde(rename = "validTo")]
    pub valid_to: Option<DateTime<Utc>>,
    #[serde(rename = "validFrom")]
    pub valid_from: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LineServiceType {
    pub name: String,
    pub uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StopPoint {
    #[serde(rename = "naptanId")]
    pub naptan_id: Option<String>,
    #[serde(rename = "platformName")]
    pub platform_name: Option<String>,
    pub indicator: Option<String>,
    #[serde(rename = "stopLetter")]
    pub stop_letter: Option<String>,
    #[serde(default)]
    pub modes: Vec<String>,
    #[serde(rename = "icsCode")]
    pub ics_code: Option<String>,
    #[serde(rename = "smsCode")]
    pub sms_code: Option<String>,
    #[serde(rename = "stopType")]
    pub stop_type: Option<String>,
    #[serde(rename = "stationNaptan")]
    pub station_naptan: Option<String>,
    #[serde(rename = "accessibilitySummary")]
    pub accessibility_summary: Option<String>,
    #[serde(rename = "hubNaptanCode")]
    pub hub_naptan_code: Option<String>,
    pub id: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "commonName")]
    pub common_name: Option<String>,
    pub distance: Option<f64>,
    #[serde(rename = "placeType")]
    pub place_type: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub status: Option<bool>,
}

// Arrival models

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Prediction {
    pub id: Option<String>,
    #[serde(rename = "operationType")]
    pub operation_type: Option<i32>,
    #[serde(rename = "vehicleId")]
    pub vehicle_id: Option<String>,
    #[serde(rename = "naptanId")]
    pub naptan_id: Option<String>,
    #[serde(rename = "stationName")]
    pub station_name: Option<String>,
    #[serde(rename = "lineId")]
    pub line_id: Option<String>,
    #[serde(rename = "lineName")]
    pub line_name: Option<String>,
    #[serde(rename = "platformName")]
    pub platform_name: Option<String>,
    pub direction: Option<String>,
    pub bearing: Option<String>,
    #[serde(rename = "destinationNaptanId")]
    pub destination_naptan_id: Option<String>,
    #[serde(rename = "destinationName")]
    pub destination_name: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(rename = "timeToStation")]
    pub time_to_station: Option<i32>,
    #[serde(rename = "currentLocation")]
    pub current_location: Option<String>,
    pub towards: Option<String>,
    #[serde(rename = "expectedArrival")]
    pub expected_arrival: Option<DateTime<Utc>>,
    #[serde(rename = "timeToLive")]
    pub time_to_live: Option<DateTime<Utc>>,
    #[serde(rename = "modeName")]
    pub mode_name: Option<String>,
    pub timing: Option<PredictionTiming>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PredictionTiming {
    // #[serde(rename = "countdownServerAdjustment")]
    // pub countdown_server_adjustment: Option<NaiveTime>,
    // pub source: Option<NaiveDateTime>,
    // pub insert: Option<NaiveDateTime>,
    pub read: Option<DateTime<Utc>>,
    pub sent: Option<DateTime<Utc>>,
    // pub received: Option<NaiveDateTime>,
}

// Station models

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Station {
    #[serde(rename = "stationUniqueId")]
    pub station_unique_id: String,
    #[serde(rename = "stationName")]
    pub station_name: String,
    #[serde(rename = "fareZones")]
    pub fare_zones: Option<String>,
    #[serde(rename = "hubNaptanCode")]
    pub hub_naptan_code: Option<String>,
    pub wifi: Option<bool>,
    #[serde(rename = "outsideStationUniqueId")]
    pub outside_station_unique_id: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    #[serde(default)]
    pub lines: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StationPoint {
    #[serde(rename = "uniqueId")]
    pub unique_id: String,
    #[serde(rename = "stationUniqueId")]
    pub station_unique_id: String,
    #[serde(rename = "areaName")]
    pub area_name: String,
    #[serde(rename = "areaId")]
    pub area_id: i32,
    pub level: i32,
    pub lat: f64,
    pub lon: f64,
    #[serde(rename = "friendlyName")]
    pub friendly_name: String,
}
