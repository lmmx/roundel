use crate::tb8::Prediction;
use crate::data::TflDataRepository;
use crate::data::api::{fetch_arrivals_for_line, get_vehicle_type_for_line};
use crate::utils::log::{LogCategory, debug_with_category, info_with_category, warn_with_category};
use js_sys::Math;

#[derive(Clone, Debug)]
pub enum VehicleType {
    Bus,
    Train,
}

#[derive(Clone, Debug)]
pub struct Vehicle {
    pub id: usize,
    pub vehicle_type: VehicleType,
    pub route_index: usize,
    pub line_id: String,
    pub position: f64,       // 0.0 to 1.0 position along route segment
    pub speed: f64,          // Movement speed
    pub direction: i8,       // 1 = forward, -1 = backward
    pub last_station: usize, // Index of last station
    pub next_station: usize, // Index of station we're heading towards
    pub lng: f64,            // Current longitude
    pub lat: f64,            // Current latitude
}

#[derive(Clone, Debug)]
pub struct Route {
    pub id: usize,
    pub name: String,
    pub line_id: String,
    pub vehicle_type: VehicleType,
    pub stations: Vec<(f64, f64)>, // Vec of (lng, lat) coordinates
}

/// Create sample routes based on TfL network
pub fn build_sample_routes() -> Vec<Route> {
    // Create a vector to hold the routes
    let mut routes = Vec::new();

    // Central Line (simplified)
    routes.push(Route {
        id: 0,
        name: "central (segment 0)".to_string(),
        line_id: "central".to_string(),
        vehicle_type: VehicleType::Train,
        stations: vec![
            // West to East: Longitude, Latitude
            (-0.2810, 51.5170), // West Ruislip
            (-0.2528, 51.5113), // Ruislip Gardens
            (-0.2194, 51.5136), // South Ruislip
            (-0.1987, 51.5202), // Northolt
            (-0.1652, 51.5259), // Greenford
            (-0.1350, 51.5210), // Perivale
            (-0.0997, 51.5152), // Hanger Lane
            (-0.0638, 51.5165), // North Acton
            (-0.0362, 51.5111), // East Acton
            (-0.0244, 51.5043), // White City
            (-0.0048, 51.5035), // Shepherd's Bush
            (-0.0125, 51.5009), // Holland Park
            (-0.0199, 51.4996), // Notting Hill Gate
            (-0.0457, 51.5068), // Queensway
            (-0.0742, 51.5113), // Lancaster Gate
            (-0.0983, 51.5142), // Marble Arch
            (-0.1280, 51.5151), // Bond Street
            (-0.1410, 51.5154), // Oxford Circus
            (-0.1687, 51.5174), // Tottenham Court Road
            (-0.1889, 51.5206), // Holborn
            (-0.1205, 51.5152), // Chancery Lane
            (-0.1025, 51.5168), // St. Paul's
            (-0.0911, 51.5155), // Bank
            (-0.0765, 51.5108), // Liverpool Street
        ],
    });

    // Northern Line (simplified)
    routes.push(Route {
        id: 1,
        name: "northern (segment 0)".to_string(),
        line_id: "northern".to_string(),
        vehicle_type: VehicleType::Train,
        stations: vec![
            // North to South
            (-0.1938, 51.6503), // High Barnet
            (-0.1932, 51.6302), // Totteridge & Whetstone
            (-0.1858, 51.6179), // Woodside Park
            (-0.1750, 51.6071), // West Finchley
            (-0.1647, 51.5998), // Finchley Central
            (-0.1534, 51.5874), // East Finchley
            (-0.1419, 51.5775), // Highgate
            (-0.1303, 51.5717), // Archway
            (-0.1123, 51.5656), // Tufnell Park
            (-0.1051, 51.5545), // Kentish Town
            (-0.1426, 51.5302), // Camden Town
            (-0.1385, 51.5248), // Mornington Crescent
            (-0.1343, 51.5287), // Euston
            (-0.1304, 51.5295), // King's Cross St. Pancras
            (-0.1231, 51.5203), // Angel
            (-0.1065, 51.5121), // Old Street
            (-0.0882, 51.5176), // Moorgate
            (-0.0911, 51.5155), // Bank
            (-0.0924, 51.5113), // London Bridge
            (-0.1002, 51.5044), // Borough
            (-0.1052, 51.4944), // Elephant & Castle
        ],
    });

    // Bus route (sample)
    routes.push(Route {
        id: 2,
        name: "88 (segment 0)".to_string(),
        line_id: "88".to_string(),
        vehicle_type: VehicleType::Bus,
        stations: vec![
            // West to East (Camden to Canning Town)
            (-0.1465, 51.5365), // Camden Town
            (-0.1325, 51.5300), // St Pancras
            (-0.1155, 51.5235), // Farringdon
            (-0.0958, 51.5181), // Barbican
            (-0.0879, 51.5155), // Moorgate
            (-0.0825, 51.5127), // Liverpool Street
            (-0.0754, 51.5101), // Aldgate
            (-0.0650, 51.5088), // Aldgate East
            (-0.0550, 51.5070), // Whitechapel
            (-0.0449, 51.5055), // Stepney Green
            (-0.0349, 51.5040), // Mile End
            (-0.0250, 51.5025), // Bow Road
            (-0.0150, 51.5010), // Bow Church
            (-0.0050, 51.4995), // Devons Road
            (0.0050, 51.4980),  // Langdon Park
            (0.0150, 51.4965),  // All Saints
            (0.0250, 51.4950),  // Poplar
            (0.0350, 51.4935),  // Blackwall
            (0.0450, 51.4920),  // East India
            (0.0550, 51.4905),  // Canning Town
        ],
    });

    routes
}

/// Build actual routes from TfL data repository
pub fn build_routes_from_tfl_data(tfl_data: &TflDataRepository) -> Vec<Route> {
    let mut routes = Vec::new();
    let mut route_id = 0;

    // Process each line with route geometries
    for (line_id, geometries) in &tfl_data.route_geometries {
        if geometries.is_empty() {
            continue;
        }

        // Determine vehicle type based on line ID and its first route sequence's mode
        let route_mode = tfl_data
            .routes
            .get(line_id)
            .and_then(|directions| directions.values().next())
            .and_then(|response| response.first())
            .map(|route_sequence| route_sequence.mode.to_lowercase())
            .unwrap_or_else(|| "train".to_string());

        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
            "Route mode for {}: {}",
            line_id, route_mode
        )));

        let vehicle_type = match route_mode.as_str() {
            "bus" => VehicleType::Bus,
            _ => VehicleType::Train,
        };

        // Process each route segment for this line
        for (segment_idx, coordinates) in geometries.iter().enumerate() {
            // Skip segments with too few coordinates
            if coordinates.len() < 2 {
                continue;
            }

            // Convert coordinates from [f64; 2] to stations format (f64, f64)
            let mut stations = Vec::new();
            for coord in coordinates {
                stations.push((coord[0], coord[1])); // lng, lat
            }

            // Create a route from this geometry
            routes.push(Route {
                id: route_id,
                name: format!("{} (segment {})", line_id, segment_idx),
                line_id: line_id.clone(),
                vehicle_type: vehicle_type.clone(),
                stations,
            });

            route_id += 1;
        }
    }

    // If no routes were created, fall back to sample routes
    if routes.is_empty() {
        warn_with_category(
            LogCategory::Simulation,
            "No valid routes found in TfL data, using sample routes",
        );
        return build_sample_routes();
    }

    routes
}

/// Initialize vehicles on the routes
pub fn initialize_vehicles(routes: &[Route]) -> Vec<Vehicle> {
    let mut vehicles = Vec::new();
    let mut id_counter = 0;

    for route in routes {
        let vehicle_count = match route.vehicle_type {
            VehicleType::Train => 2, // 2 trains per route
            VehicleType::Bus => 2,   // 2 buses per route
        };

        // Create vehicles distributed along the route
        for i in 0..vehicle_count {
            // Determine starting positions and directions
            let (last_station, next_station, direction) = if i % 2 == 0 {
                // Forward direction
                (0, 1, 1)
            } else {
                // Backward direction
                (route.stations.len() - 1, route.stations.len() - 2, -1)
            };

            // Get station coordinates
            let (start_lng, start_lat) = route.stations[last_station];

            // Create vehicle
            vehicles.push(Vehicle {
                id: id_counter,
                vehicle_type: route.vehicle_type.clone(),
                route_index: route.id,
                line_id: route.line_id.clone(),
                position: Math::random(), // Random position along segment
                speed: 0.005 + Math::random() * 0.05, // Random speed
                direction,
                last_station,
                next_station,
                lng: start_lng,
                lat: start_lat,
            });

            id_counter += 1;
        }
    }

    vehicles
}

// --- Real arrivals data based simulation ---

/// Build routes and vehicles from real-time arrival data
pub async fn build_real_time_vehicles() -> Result<Vec<Vehicle>, String> {
    info_with_category(
        LogCategory::Simulation,
        "Building vehicles from real-time arrival data",
    );

    let mut vehicles = Vec::new();
    let mut id_counter = 0;

    // Try to fetch for each supported line - focus on a few key lines for the MVP
    let primary_lines = [
        "victoria", "piccadilly", "northern", "jubilee", "central",
        "district", "bakerloo", "waterloo-city", "circle", "hammersmith-city",
        "metropolitan", "dlr", "elizabeth", "tram"
    ];

    for line_id in primary_lines.iter() {
        match fetch_arrivals_for_line(line_id).await {
            Ok(predictions) => {
                debug_with_category(
                    LogCategory::Simulation,
                    &format!("Processing {} arrivals for {}", predictions.len(), line_id),
                );

                // Group predictions by vehicle ID to avoid duplicates
                let mut vehicle_map = std::collections::HashMap::<String, Prediction>::new();

                for prediction in predictions {
                    // Skip if no vehicle ID or coordinates
                    if prediction.vehicle_id.is_none() || prediction.time_to_station.is_none() {
                        continue;
                    }

                    let vehicle_id = prediction.vehicle_id.clone().unwrap();

                    // Use this prediction if we haven't seen this vehicle before
                    // or if it's closer to arriving than the previous one
                    if !vehicle_map.contains_key(&vehicle_id) ||
                       prediction.time_to_station.unwrap() < vehicle_map.get(&vehicle_id).unwrap().time_to_station.unwrap() {
                        vehicle_map.insert(vehicle_id, prediction);
                    }
                }

                // Convert each unique vehicle to our simulation model
                for (_, prediction) in vehicle_map {
                    // Skip if missing key data
                    if prediction.line_id.is_none() ||
                       prediction.time_to_station.is_none() ||
                       prediction.current_location.is_none() {
                        continue;
                    }

                    // This would be better with actual coordinates, but for MVP just
                    // use a random position around London (will be improved later)
                    let base_lon = -0.1278; // London center
                    let base_lat = 51.5074;

                    // Add some randomness - spread vehicles around London
                    let random_offset = (Math::random() - 0.5) * 0.1;
                    let lon = base_lon + random_offset;
                    let lat = base_lat + random_offset;

                    // Convert the time to station into a position (0.0 to 1.0)
                    // Closer to arrival = closer to station (higher position value)
                    let time_to_station = prediction.time_to_station.unwrap() as f64;
                    let max_time = 600.0; // 10 minutes as max
                    let position = (max_time - time_to_station.min(max_time)) / max_time;

                    // Determine direction (1 = toward, -1 = away)
                    let direction = 1; // Default to moving toward station

                    // Get line ID
                    let line_id = prediction.line_id.unwrap();

                    // Create a vehicle
                    vehicles.push(Vehicle {
                        id: id_counter,
                        vehicle_type: get_vehicle_type_for_line(&line_id),
                        route_index: 0, // We'll set this correctly when we build routes
                        line_id: line_id,
                        position,
                        speed: 0.01, // Default speed
                        direction,
                        last_station: 0,
                        next_station: 1,
                        lng: lon,
                        lat: lat,
                    });

                    id_counter += 1;
                }
            },
            Err(e) => {
                warn_with_category(
                    LogCategory::Simulation,
                    &format!("Failed to fetch arrivals for {}: {}", line_id, e),
                );
                // Continue with other lines if one fails
                continue;
            }
        }
    }

    // If we got zero vehicles, return an error so we can fall back to sample data
    if vehicles.is_empty() {
        return Err("No real-time vehicles found".to_string());
    }

    info_with_category(
        LogCategory::Simulation,
        &format!("Built {} vehicles from real-time data", vehicles.len()),
    );

    Ok(vehicles)
}

/// Create simple routes for real-time vehicles
pub fn create_simple_routes_for_real_time() -> Vec<Route> {
    // For the MVP, just create one route per line type
    // This is a simplified approach - in a full implementation,
    // you'd want to use actual route data

    let primary_lines = [
        "victoria", "piccadilly", "northern", "jubilee", "central",
        "district", "bakerloo", "waterloo-city", "circle", "hammersmith-city",
        "metropolitan", "dlr", "elizabeth", "tram"
    ];

    let mut routes = Vec::new();

    for (i, line_id) in primary_lines.iter().enumerate() {
        // Create a simple route with just two points
        // Central London coordinates
        routes.push(Route {
            id: i,
            name: format!("{} (segment 0)", line_id),
            line_id: line_id.to_string(),
            vehicle_type: get_vehicle_type_for_line(line_id),
            stations: vec![
                (-0.1278, 51.5074),  // London center
                (-0.1178, 51.5174),  // A short distance away
            ],
        });
    }

    routes
}
