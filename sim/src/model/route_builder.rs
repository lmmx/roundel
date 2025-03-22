// src/model/route_builder.rs

use crate::model::Route;
use crate::model::geo::{GeoProjection, generate_route_path};
use js_sys::Math;

// Constant for canvas dimensions
const CANVAS_WIDTH: f32 = 1000.0;
const CANVAS_HEIGHT: f32 = 1000.0;

/// Build train routes that approximately follow the pattern of London tube lines
pub fn build_random_train_routes() -> Vec<Route> {
    let projection = GeoProjection::london_centered(CANVAS_WIDTH, CANVAS_HEIGHT);
    let mut train_routes = Vec::new();

    // Central London coordinates (approximately)
    let central_london = (51.51, -0.12); // Roughly Oxford Circus
    let central_xy = projection.project(central_london.0, central_london.1);

    // Approximate directions of major tube lines from central London
    let directions = [
        (51.59, -0.33), // Northwest (toward Harrow)
        (51.69, 0.11),  // Northeast (toward Epping)
        (51.49, -0.22), // West (toward Hammersmith)
        (51.54, 0.08),  // East (toward Barking)
        (51.62, -0.28), // North (toward Edgware)
        (51.40, -0.19), // South (toward Morden)
        (51.47, -0.48), // Southwest (toward Heathrow)
        (51.65, -0.14), // North-northeast (toward Cockfosters)
        (51.58, -0.02), // Northeast (toward Walthamstow)
        (51.46, -0.11), // South (toward Brixton)
    ];

    // Create routes for each direction
    for (_i, &(end_lat, end_lng)) in directions.iter().enumerate().take(10) {
        let end_xy = projection.project(end_lat, end_lng);

        // Generate a path with some randomness to simulate curves in the tracks
        let num_stations = 4 + (Math::random() * 4.0) as usize; // 4-8 stations
        let stations = generate_route_path(
            central_xy,
            end_xy,
            num_stations,
            0.2, // moderate randomness
        );

        train_routes.push(Route { stations });
    }

    train_routes
}

/// Build bus routes that follow a more complex pattern
pub fn build_random_bus_routes() -> Vec<Route> {
    let projection = GeoProjection::london_centered(CANVAS_WIDTH, CANVAS_HEIGHT);
    let mut bus_routes = Vec::new();

    // Central London coordinates
    let central_london = (51.51, -0.12);

    // Create a mix of:
    // 1. Radial routes (from center outward)
    // 2. Orbital routes (circular)
    // 3. Cross-town routes (north-south or east-west)

    // 1. Radial routes (30 routes)
    for _ in 0..30 {
        // Random angle from center
        let angle = Math::random() as f32 * 2.0 * std::f32::consts::PI;
        let distance = 0.1 + Math::random() as f32 * 0.2; // 10-30km approximately

        // Calculate end point
        let end_lat = central_london.0 + distance * angle.sin();
        let end_lng = central_london.1 + distance * angle.cos();

        // Project to canvas
        let start_xy = projection.project(central_london.0, central_london.1);
        let end_xy = projection.project(end_lat, end_lng);

        // Generate path with waypoints
        let stations = generate_route_path(
            start_xy,
            end_xy,
            5 + (Math::random() * 7.0) as usize, // 5-12 stations
            0.3,                                 // moderate randomness
        );

        bus_routes.push(Route { stations });
    }

    // 2. Orbital routes (20 routes)
    for i in 0..20 {
        let radius = 0.03 + (i as f32 / 20.0) * 0.15; // 3km to 18km approx
        let start_angle = Math::random() as f32 * 2.0 * std::f32::consts::PI;

        let mut stations = Vec::new();
        let num_segments = 8 + (Math::random() * 8.0) as usize; // 8-16 segments

        for j in 0..=num_segments {
            let angle = start_angle + (j as f32 / num_segments as f32) * 2.0 * std::f32::consts::PI;
            let lat = central_london.0 + radius * angle.sin();
            let lng = central_london.1 + radius * angle.cos();
            let xy = projection.project(lat, lng);

            // Add some randomness to the orbital path
            let rand_offset = radius * 0.1; // 10% of radius
            let x_offset = (Math::random() as f32 * 2.0 - 1.0) * rand_offset * projection.scale;
            let y_offset = (Math::random() as f32 * 2.0 - 1.0) * rand_offset * projection.scale;

            stations.push((xy.0 + x_offset, xy.1 + y_offset));
        }

        // Close the loop for orbital routes
        if !stations.is_empty() {
            stations.push(stations[0]);
        }

        bus_routes.push(Route { stations });
    }

    // 3. Cross-town routes (50 routes)
    for _ in 0..50 {
        // Decide whether this is north-south or east-west
        let is_east_west = Math::random() > 0.5;

        // Create a start and end point
        let offset = -0.1 + Math::random() as f32 * 0.2; // -10km to +10km from center

        let (start_lat, start_lng, end_lat, end_lng) = if is_east_west {
            // East-west route
            (
                central_london.0 + offset,
                central_london.1 - 0.15 - Math::random() as f32 * 0.1, // West
                central_london.0 + offset,
                central_london.1 + 0.15 + Math::random() as f32 * 0.1, // East
            )
        } else {
            // North-south route
            (
                central_london.0 - 0.15 - Math::random() as f32 * 0.1, // South
                central_london.1 + offset,
                central_london.0 + 0.15 + Math::random() as f32 * 0.1, // North
                central_london.1 + offset,
            )
        };

        let start_xy = projection.project(start_lat, start_lng);
        let end_xy = projection.project(end_lat, end_lng);

        // Generate path with waypoints
        let stations = generate_route_path(
            start_xy,
            end_xy,
            6 + (Math::random() * 10.0) as usize, // 6-16 stations
            0.25,                                 // moderate randomness
        );

        bus_routes.push(Route { stations });
    }

    bus_routes
}

// COMMENTED CODE FOR FUTURE IMPLEMENTATION:
// This shows how you would parse and use the real TSV data

/*
/// Parse TSV file content into route data
fn parse_tsv_data(content: &str) -> Vec<(String, String, (f32, f32), String, (f32, f32))> {
    let mut routes = Vec::new();

    // Skip header line
    let lines = content.lines().skip(1);

    for line in lines {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() >= 5 {
            let route_name = fields[0].to_string();
            let start_name = fields[1].to_string();

            // Try to parse start coordinates
            let start_coords = match parse_coordinate(fields[2]) {
                Ok(coords) => coords,
                Err(_) => continue, // Skip invalid entries
            };

            let end_name = fields[3].to_string();

            // Try to parse end coordinates
            let end_coords = match parse_coordinate(fields[4]) {
                Ok(coords) => coords,
                Err(_) => continue, // Skip invalid entries
            };

            routes.push((route_name, start_name, start_coords, end_name, end_coords));
        }
    }

    routes
}

/// Build train routes from the real TSV data
pub fn build_real_train_routes() -> Vec<Route> {
    // Step 1: Load the TSV content (this would need a different approach in WASM)
    // For example, using JavaScript to fetch the file and pass it to Rust
    let tsv_content = match fetch_tsv_file("public/tube_routes.tsv") {
        Some(content) => content,
        None => {
            console::log_1(&"Failed to load tube routes, using random fallback".into());
            return build_random_train_routes();
        }
    };

    // Step 2: Parse the TSV data
    let route_data = parse_tsv_data(&tsv_content);

    // Step 3: Create the projection
    let projection = GeoProjection::london_centered(CANVAS_WIDTH, CANVAS_HEIGHT);

    // Step 4: Build the routes
    let mut train_routes = Vec::new();

    for (route_name, _, start_coords, _, end_coords) in route_data {
        // Project coordinates to canvas space
        let start_xy = projection.project(start_coords.0, start_coords.1);
        let end_xy = projection.project(end_coords.0, end_coords.1);

        // Generate intermediate points to make a realistic path
        let stations = generate_route_path(
            start_xy,
            end_xy,
            3, // Fewer waypoints for trains
            0.2 // moderate randomness
        );

        train_routes.push(Route { stations });

        // Optionally log the route being created
        console::log_1(&format!("Created train route: {}", route_name).into());
    }

    train_routes
}

/// Build bus routes from the real TSV data
pub fn build_real_bus_routes() -> Vec<Route> {
    // Similar approach to build_real_train_routes
    // ...

    // For now, return random routes
    build_random_bus_routes()
}

/// Fetch TSV file content (would need JavaScript interop)
fn fetch_tsv_file(path: &str) -> Option<String> {
    // This is a placeholder - in a real implementation you would:
    // 1. Use JavaScript to fetch the file
    // 2. Pass the content to Rust via wasm_bindgen

    None // For now, always return None so we fall back to random routes
}
*/
