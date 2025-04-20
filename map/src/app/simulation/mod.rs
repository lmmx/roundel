use crate::app::simulation::model::build_routes_from_tfl_data;
use crate::data::TflDataRepository;
use crate::data::line_definitions::get_line_color;
use crate::utils::geojson::{new_geojson_source, new_point_feature, to_js_value};
use crate::utils::log::{self, LogCategory, with_context};
use js_sys::{Object, Reflect};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::window;

mod model;
mod state;

pub use model::{VehicleType, build_sample_routes, initialize_vehicles};
pub use state::{
    SimulationState, get_animation_frame_id, get_vehicle_count, initialize_state, is_paused,
    set_animation_frame_id, toggle_pause, with_simulation_state, with_simulation_state_ref,
};

// Real arrivals data simulation
// ------------------------------

// Replace or update your initialize_simulation function
pub fn initialize_simulation(tfl_data: Option<TflDataRepository>) {
    with_context("initialize_simulation", LogCategory::Simulation, |logger| {
        logger.info("Initializing vehicle simulation with real-time data...");

        // Set a global flag to track simulation visibility
        let js_code = r#"
        window.simulationVisible = true;
        console.log('Set window.simulationVisible = true');
        "#;
        let _ = js_sys::eval(js_code);

        // First try to get real-time data
        use_real_time_data(tfl_data.clone());

        logger.info("Simulation initialization requested");
    });
}

// Function to attempt using real-time data
fn use_real_time_data(tfl_data: Option<TflDataRepository>) {
    with_context("use_real_time_data", LogCategory::Simulation, |logger| {
        logger.info("Attempting to use real-time vehicle data");

        // Spawn a local future to fetch the data
        wasm_bindgen_futures::spawn_local(async move {
            match model::build_real_time_vehicles().await {
                Ok(vehicles) => {
                    log::info_with_category(
                        LogCategory::Simulation,
                        &format!("Successfully built {} real-time vehicles", vehicles.len()),
                    );

                    // Create routes for these vehicles
                    let routes = model::create_simple_routes_for_real_time();

                    // Assign routes to vehicles
                    let mut route_mapped_vehicles = vehicles;
                    for vehicle in &mut route_mapped_vehicles {
                        // Find a route matching this vehicle's line_id
                        for route in &routes {
                            if route.line_id == vehicle.line_id {
                                vehicle.route_index = route.id;
                                break;
                            }
                        }
                    }

                    // Initialize state with real-time data
                    initialize_state(routes, route_mapped_vehicles);

                    // Register with MapLibre and start animation
                    register_vehicle_layers();
                    start_animation_loop();

                    log::info_with_category(
                        LogCategory::Simulation,
                        "Real-time simulation initialized successfully",
                    );
                },
                Err(e) => {
                    // Fall back to sample/TfL data
                    log::warn_with_category(
                        LogCategory::Simulation,
                        &format!("Failed to build real-time vehicles: {}. Falling back to static data.", e),
                    );
                    use_fallback_data(tfl_data);
                }
            }
        });
    });
}

// Fallback to existing data methods
fn use_fallback_data(tfl_data: Option<TflDataRepository>) {
    with_context("use_fallback_data", LogCategory::Simulation, |logger| {
        logger.info("Using fallback sample/TfL data for simulation");

        // Build routes from real TfL data if available, otherwise use sample routes
        let routes = match tfl_data {
            Some(repo) => build_routes_from_tfl_data(&repo),
            None => {
                logger.warn("No TfL data provided, using sample routes");
                build_sample_routes()
            }
        };

        // Initialize vehicles on those routes
        let vehicles = initialize_vehicles(&routes);

        // Store in global state
        initialize_state(routes, vehicles);

        // Register with MapLibre and start animation
        register_vehicle_layers();
        start_animation_loop();

        logger.info("Fallback simulation initialized");
    });
}


// MapLibre integration components
// ------------------------------

/// Expose initialization function globally
// Expose Rust functions to JavaScript
pub fn expose_simulation_functions(tfl_data: Option<TflDataRepository>) -> Result<(), JsValue> {
    with_context(
        "expose_simulation_functions",
        LogCategory::Simulation,
        |logger| {
            logger.info("Exposing simulation functions to JavaScript");

            // Clone tfl_data so the closure can be called more than once.
            let tfl_data_for_closure = tfl_data.clone();
            // Create initialize function
            let init_closure = Closure::wrap(Box::new({
                let tfl_data_inner = tfl_data_for_closure;
                move || {
                    log::info_with_category(
                        LogCategory::Simulation,
                        "rust_initialize_simulation called from JS",
                    );
                    initialize_simulation(tfl_data_inner.clone());
                }
            }) as Box<dyn FnMut()>);

            // Create toggle function
            let toggle_closure = Closure::wrap(Box::new(|| {
                log::info_with_category(
                    LogCategory::Simulation,
                    "rust_toggle_simulation called from JS",
                );
                toggle_simulation();
            }) as Box<dyn FnMut()>);

            // Clone tfl_data for reset closure too
            let tfl_data_for_reset = tfl_data.clone();
            // Create reset function
            let reset_closure = Closure::wrap(Box::new({
                let tfl_data_inner = tfl_data_for_reset;
                move || {
                    log::info_with_category(
                        LogCategory::Simulation,
                        "rust_reset_simulation called from JS",
                    );
                    reset_simulation(tfl_data_inner.clone());
                }
            }) as Box<dyn FnMut()>);

            // Set them on the window object
            if let Some(window) = window() {
                js_sys::Reflect::set(
                    &window,
                    &JsValue::from_str("rust_initialize_simulation"),
                    init_closure.as_ref(),
                )
                .expect("Could not set rust_initialize_simulation");

                js_sys::Reflect::set(
                    &window,
                    &JsValue::from_str("rust_toggle_simulation"),
                    toggle_closure.as_ref(),
                )
                .expect("Could not set rust_toggle_simulation");

                js_sys::Reflect::set(
                    &window,
                    &JsValue::from_str("rust_reset_simulation"),
                    reset_closure.as_ref(),
                )
                .expect("Could not set rust_reset_simulation");

                logger.info("Simulation functions exposed to JavaScript");
            }

            // Leak the closures (they will live for the lifetime of the page)
            init_closure.forget();
            toggle_closure.forget();
            reset_closure.forget();

            Ok(())
        },
    )
}

// SIMULATION FUNCTIONS
// -------------------

// /// Initialize the vehicle simulation
// pub fn initialize_simulation(tfl_data: Option<TflDataRepository>) {
//     with_context("initialize_simulation", LogCategory::Simulation, |logger| {
//         logger.info("Initializing vehicle simulation...");
// 
//         // Set a global flag to track simulation visibility
//         let js_code = r#"
//         window.simulationVisible = true;
//         console.log('Set window.simulationVisible = true');
//         "#;
//         let _ = js_sys::eval(js_code);
// 
//         // Build routes from real TfL data if available, otherwise use sample routes
//         let routes = match tfl_data {
//             Some(repo) => build_routes_from_tfl_data(&repo),
//             None => {
//                 logger.warn("No TfL data provided, using sample routes");
//                 build_sample_routes()
//             }
//         };
// 
//         // Initialize vehicles on those routes
//         let vehicles = initialize_vehicles(&routes);
// 
//         // Store in global state
//         initialize_state(routes, vehicles);
// 
//         // Register with MapLibre and start animation
//         register_vehicle_layers();
//         start_animation_loop();
// 
//         logger.info("Simulation initialized");
//     });
// }

/// Register vehicle layers with MapLibre GL
fn register_vehicle_layers() {
    with_context(
        "register_vehicle_layers",
        LogCategory::Simulation,
        |logger| {
            logger.info("Registering vehicle layers with MapLibre");

            // Get the map instance from window
            if let Some(window) = window() {
                if let Ok(map_instance) =
                    js_sys::Reflect::get(&window, &JsValue::from_str("mapInstance"))
                {
                    let map: crate::maplibre::bindings::Map = map_instance.into();

                    // Check if source already exists by checking if layer exists
                    if map.get_layer("buses-layer").is_none() {
                        // Create an empty source using our utility function
                        let geojson_source = new_geojson_source(Vec::new());

                        // Serialize to JsValue
                        match to_js_value(&geojson_source) {
                            Ok(source_js) => {
                                // Add the source
                                map.add_source("vehicles-source", &source_js);

                                // Create and add bus layer
                                let bus_layer = create_vehicle_layer("buses-layer", "Bus");
                                map.add_layer(&bus_layer);

                                // Create and add train layer
                                let train_layer = create_vehicle_layer("trains-layer", "Train");
                                map.add_layer(&train_layer);

                                logger.info("Vehicle layers successfully added");
                            }
                            Err(err) => {
                                logger
                                    .error(&format!("Failed to create GeoJSON source: {:?}", err));
                            }
                        }
                    } else {
                        logger.info("Vehicle layers already exist, skipping creation");
                    }
                } else {
                    logger.error("Could not get mapInstance from window");
                }
            } else {
                logger.error("Window object not available");
            }
        },
    )
}

/// Helper function to create a vehicle layer specification
fn create_vehicle_layer(id: &str, vehicle_type: &str) -> JsValue {
    let layer = Object::new();

    // Set basic properties
    Reflect::set(&layer, &JsValue::from_str("id"), &JsValue::from_str(id)).unwrap();
    Reflect::set(
        &layer,
        &JsValue::from_str("type"),
        &JsValue::from_str("circle"),
    )
    .unwrap();
    Reflect::set(
        &layer,
        &JsValue::from_str("source"),
        &JsValue::from_str("vehicles-source"),
    )
    .unwrap();

    // Add filter for vehicle type
    let filter = js_sys::Array::new();
    filter.push(&JsValue::from_str("=="));

    let get_expr = js_sys::Array::new();
    get_expr.push(&JsValue::from_str("get"));
    get_expr.push(&JsValue::from_str("vehicleType"));

    filter.push(&get_expr);
    filter.push(&JsValue::from_str(vehicle_type));

    Reflect::set(&layer, &JsValue::from_str("filter"), &filter).unwrap();

    // Add paint properties
    let paint = Object::new();

    Reflect::set(
        &paint,
        &JsValue::from_str("circle-radius"),
        &JsValue::from_f64(6.0),
    )
    .unwrap();
    // Use the color property directly
    let color_expr = js_sys::Array::new();
    color_expr.push(&JsValue::from_str("get"));
    color_expr.push(&JsValue::from_str("lineColor"));

    Reflect::set(&paint, &JsValue::from_str("circle-color"), &color_expr).unwrap();
    Reflect::set(
        &paint,
        &JsValue::from_str("circle-stroke-color"),
        &JsValue::from_str("#FFFFFF"),
    )
    .unwrap();
    Reflect::set(
        &paint,
        &JsValue::from_str("circle-stroke-width"),
        &JsValue::from_f64(2.0),
    )
    .unwrap();

    Reflect::set(&layer, &JsValue::from_str("paint"), &paint).unwrap();

    layer.into()
}

/// Start the animation loop for vehicle movement with throttled updates
fn start_animation_loop() {
    with_context("start_animation_loop", LogCategory::Simulation, |logger| {
        logger.debug("Starting throttled animation loop for vehicle movement");

        // Set a fixed interval for updates instead of using requestAnimationFrame
        let update_interval_ms = 1000 / 30; // 15 FPS

        // Create a JavaScript setInterval to handle the animation loop
        let js_code = format!(
            r#"
            // Clear any existing interval
            if (window.__rustAnimIntervalId) {{
                clearInterval(window.__rustAnimIntervalId);
            }}

            // Set new interval for animation updates
            window.__rustAnimIntervalId = setInterval(function() {{
                // Only call the Rust function if defined
                if (typeof window.rust_animation_tick === 'function') {{
                    window.rust_animation_tick();
                }}
            }}, {});

            // Return the interval ID
            window.__rustAnimIntervalId;
        "#,
            update_interval_ms
        );

        // Execute the JavaScript to start the interval
        let interval_id = js_sys::eval(&js_code)
            .unwrap_or(JsValue::from_f64(0.0))
            .as_f64()
            .unwrap_or(0.0) as i32;

        // Store the interval ID where animation frame ID would normally go
        set_animation_frame_id(interval_id);

        // Create the animation tick function
        let tick_closure = Closure::wrap(Box::new(move || {
            // Process a single animation frame
            let should_continue = with_simulation_state(|sim_state| {
                if !sim_state.is_paused {
                    // Update vehicle positions
                    update_vehicle_positions(sim_state);

                    // Update MapLibre with new positions
                    update_maplibre_vehicles(sim_state);
                }

                // Return true to keep the interval running
                // The actual pause state is checked on each tick
                true
            });

            // If the simulation should be completely stopped (not just paused)
            if !should_continue {
                // Clear the interval
                let clear_js = r#"
                    if (window.__rustAnimIntervalId) {
                        clearInterval(window.__rustAnimIntervalId);
                        window.__rustAnimIntervalId = null;
                    }
                "#;
                let _ = js_sys::eval(clear_js);
            }
        }) as Box<dyn FnMut()>);

        // Store the tick function in the window object
        if let Some(window) = window() {
            let _ = js_sys::Reflect::set(
                &window,
                &JsValue::from_str("rust_animation_tick"),
                tick_closure.as_ref(),
            );

            // Forget the closure so it stays valid
            tick_closure.forget();

            logger.info(&format!(
                "Animation loop started with throttled updates at {} FPS",
                1000 / update_interval_ms
            ));
        } else {
            logger.error("No global window exists, cannot start animation loop");
        }
    })
}

/// Request a new animation frame
fn request_animation_frame() {
    let animation_callback = Closure::wrap(Box::new(move || {
        let should_continue = with_simulation_state(|sim_state| {
            if !sim_state.is_paused {
                // Update vehicle positions
                update_vehicle_positions(sim_state);

                // Update MapLibre with new positions
                update_maplibre_vehicles(sim_state);
            }

            // Return whether we're paused to determine if we should request another frame
            !sim_state.is_paused
        });

        // Request next animation frame if not paused
        if should_continue {
            request_animation_frame();
        }
    }) as Box<dyn FnMut()>);

    // Request animation frame
    if let Some(window) = window() {
        match window.request_animation_frame(animation_callback.as_ref().unchecked_ref()) {
            Ok(id) => {
                // Store the animation frame ID
                set_animation_frame_id(id);

                // Forget the closure to keep it alive
                animation_callback.forget();
            }
            Err(err) => {
                log::error_with_category(
                    LogCategory::Simulation,
                    &format!("Failed to request animation frame: {:?}", err),
                );
            }
        }
    } else {
        log::error_with_category(
            LogCategory::Simulation,
            "No global window exists, cannot request animation frame",
        );
    }
}

/// Update positions of all vehicles based on their speed and direction
fn update_vehicle_positions(sim_state: &mut SimulationState) {
    // This function is called less frequently now - adjust logging frequency
    static mut POSITION_UPDATE_COUNTER: u32 = 0;
    let should_log = unsafe {
        POSITION_UPDATE_COUNTER += 1;
        POSITION_UPDATE_COUNTER % 75 == 0 // Log roughly every 5 seconds (assuming 15fps)
    };

    if should_log {
        log::debug_with_category(
            LogCategory::Simulation,
            &format!(
                "Updating positions for {} vehicles",
                sim_state.vehicles.len()
            ),
        );
    }

    for vehicle in &mut sim_state.vehicles {
        // Get the current route
        let route = &sim_state.routes[vehicle.route_index];

        // Update position along segment
        vehicle.position += vehicle.speed;

        // Check if we've reached the next station
        while vehicle.position >= 1.0 {
            vehicle.position -= 1.0;
            vehicle.last_station = vehicle.next_station;

            // Determine next station based on direction
            let next_station = (vehicle.last_station as i32) + (vehicle.direction as i32);

            // Check if we need to reverse direction (reached end of line)
            if next_station < 0 || next_station >= route.stations.len() as i32 {
                // Reverse direction
                vehicle.direction *= -1;

                // Recalculate next station
                let next_station = (vehicle.last_station as i32) + (vehicle.direction as i32);
                vehicle.next_station = next_station as usize;
            } else {
                vehicle.next_station = next_station as usize;
            }
        }

        // Interpolate position between stations
        let (last_lng, last_lat) = route.stations[vehicle.last_station];
        let (next_lng, next_lat) = route.stations[vehicle.next_station];

        // Linear interpolation based on position (0.0 to 1.0)
        vehicle.lng = last_lng + (next_lng - last_lng) * vehicle.position;
        vehicle.lat = last_lat + (next_lat - last_lat) * vehicle.position;
    }
}

/// Update MapLibre with the current vehicle positions
fn update_maplibre_vehicles(sim_state: &SimulationState) {
    // This function is called less frequently now - adjust logging frequency
    static mut MAPLIBRE_UPDATE_COUNTER: u32 = 0;
    let should_log = unsafe {
        MAPLIBRE_UPDATE_COUNTER += 1;
        MAPLIBRE_UPDATE_COUNTER % 150 == 0 // Log roughly every 10 seconds (assuming 15fps)
    };

    if should_log {
        log::debug_with_category(
            LogCategory::Simulation,
            &format!(
                "Updating MapLibre with {} vehicle positions",
                sim_state.vehicles.len()
            ),
        );
    }

    // Create features for all vehicles
    let features: Vec<_> = sim_state
        .vehicles
        .iter()
        .map(|vehicle| {
            let vehicle_type = match vehicle.vehicle_type {
                VehicleType::Bus => "Bus",
                VehicleType::Train => "Train",
            };

            // Determine color based on vehicle type
            let color = match vehicle.vehicle_type {
                VehicleType::Bus => "#0000FF".to_string(), // Blue for buses
                VehicleType::Train => {
                    // For trains, get color from the line_id
                    get_line_color(&vehicle.line_id)
                }
            };

            // Create properties for this vehicle
            let properties = serde_json::json!({
                "id": vehicle.id,
                "vehicleType": vehicle_type,
                "lineId": vehicle.line_id,
                "lineColor": color
            });

            // Create a point feature
            new_point_feature(vehicle.lng, vehicle.lat, properties)
        })
        .collect();

    // Create the GeoJSON source
    let geojson_source = new_geojson_source(features);

    // Try to get the map instance and update the source
    if let Some(window) = window() {
        if let Ok(_map_instance) = js_sys::Reflect::get(&window, &JsValue::from_str("mapInstance"))
        {
            // Check if the source exists using JS eval for now
            let has_source = js_sys::eval(
                "window.mapInstance && window.mapInstance.getSource('vehicles-source') != null",
            )
            .unwrap_or(JsValue::from_bool(false))
            .as_bool()
            .unwrap_or(false);

            if has_source {
                // Serialize to JsValue - use .data to get just the FeatureCollection
                match to_js_value(&geojson_source.data) {
                    Ok(data) => {
                        // Update the source data
                        let js_code = r#"
                            window.mapInstance.getSource('vehicles-source').setData(arguments[0]);
                        "#;

                        if let Err(err) = js_sys::Function::new_with_args("data", js_code)
                            .call1(&JsValue::NULL, &data)
                        {
                            if should_log {
                                log::error_with_category(
                                    LogCategory::Simulation,
                                    &format!("Failed to update vehicle source: {:?}", err),
                                );
                            }
                        }
                    }
                    Err(err) => {
                        if should_log {
                            log::error_with_category(
                                LogCategory::Simulation,
                                &format!("Failed to serialize vehicle data: {:?}", err),
                            );
                        }
                    }
                }
            } else if should_log {
                log::warn_with_category(
                    LogCategory::Simulation,
                    "Vehicles source does not exist yet",
                );
            }
        }
    }
}

/// Toggle the simulation pause state
pub fn toggle_simulation() {
    with_context("toggle_simulation", LogCategory::Simulation, |logger| {
        let is_now_paused = toggle_pause();

        if is_now_paused {
            logger.info("Pausing simulation");
        } else {
            logger.info("Resuming simulation");
        }
    })
}

/// Reset the simulation
pub fn reset_simulation(tfl_data: Option<TflDataRepository>) {
    with_context("reset_simulation", LogCategory::Simulation, |logger| {
        logger.info("Resetting simulation...");

        // Cancel current animation interval if one is active
        if let Some(id) = get_animation_frame_id() {
            let clear_js = format!(
                r#"
                if (window.__rustAnimIntervalId) {{
                    clearInterval(window.__rustAnimIntervalId);
                    window.__rustAnimIntervalId = null;
                    console.log("Cleared animation interval: {}")
                }}
            "#,
                id
            );
            let _ = js_sys::eval(&clear_js);

            logger.debug(&format!("Canceled animation interval ID: {}", id));
        }

        // Reset state and recreate everything
        logger.debug("Re-initializing simulation from scratch");
        initialize_simulation(tfl_data);

        logger.info("Simulation reset complete");
    })
}
