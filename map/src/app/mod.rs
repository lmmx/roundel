//! # Application Module
//!
//! Main application components for the TfL Simulation.

use dioxus::prelude::*;

mod canvas;
mod key_panel;
mod layer_panel;
mod line_css;
pub mod simulation;
mod simulation_panel; // New module for vehicle simulation

use crate::app::line_css::LineCss;
use crate::data::TflDataRepository;
use crate::data::line_definitions::get_line_color;
use crate::maplibre::helpers;
use crate::maplibre::helpers::{create_circle_layer, create_label_layer, create_line_layer};
use crate::utils::log::{self, LogCategory, with_context};
use canvas::Canvas;
use key_panel::KeyPanel;
use layer_panel::LayerPanel;
use simulation_panel::SimulationPanel;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::window;

// If you have images or CSS as assets, define them with Dioxus' asset! macro
const FAVICON: Asset = asset!("/assets/favicon.ico");
const LOGO_SVG: Asset = asset!("/assets/header.svg");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TFL_CSS: Asset = asset!("/assets/tfl.css");
const KEY_CSS: Asset = asset!("/assets/key.css");
const SIM_CSS: Asset = asset!("/assets/simulation.css");
const LAYER_CSS: Asset = asset!("/assets/layerswitcher.css");

/// Model to track layer visibility.
///
/// This structure tracks which layers are visible in the TfL network map.
#[derive(Clone, Copy, PartialEq)]
pub struct TflLayers {
    /// Underground/tube lines
    pub tube: bool,
    /// Overground rail services
    pub overground: bool,
    /// Docklands Light Railway
    pub dlr: bool,
    /// Elizabeth Line (Crossrail)
    pub elizabeth_line: bool,
    /// Thameslink
    pub thameslink: bool,
    /// Bus routes
    pub buses: bool,
    /// Tram services
    pub trams: bool,
    /// Emirates Air Line cable car
    pub cable_car: bool,
    /// Place labels
    pub labels: bool,
    /// Station markers and labels
    pub stations: bool,
    /// Depot locations
    pub depots: bool,
    /// Vehicle simulation
    pub simulation: bool,
}

impl Default for TflLayers {
    fn default() -> Self {
        Self {
            tube: true,
            overground: true,
            dlr: true,
            elizabeth_line: true,
            thameslink: true,
            buses: false,
            trams: true,
            cable_car: true,
            labels: false,
            stations: true,
            depots: false,
            simulation: false, // Simulation disabled by default
        }
    }
}

/// Main application component.
///
/// This is the root component of the TfL Simulation application.
#[component]
pub fn app() -> Element {
    let mut show_layers_panel = use_signal(|| false);
    let mut show_key_panel = use_signal(|| false);
    let mut show_simulation_panel = use_signal(|| false);
    let mut simulation_initialized = use_signal(|| false);
    let mut simulation_is_paused = use_signal(|| true);
    let mut vehicle_count = use_signal(|| Option::<usize>::None);
    let mut load_bus_routes = use_signal(|| false);
    let layers = use_signal(TflLayers::default);
    let mut tfl_data = use_signal(TflDataRepository::default);

    use_future(move || async move {
        with_context("app::load_tfl_data", LogCategory::App, |logger| {
            logger.info("Loading TfL station and platform data");

            // Only load if not already loaded
            if !tfl_data.read().is_loaded {
                logger.info("Initializing TfL data repository");

                // Use spawn_local for the async operation, but don't use logger inside
                wasm_bindgen_futures::spawn_local(async move {
                    let should_load_buses = *load_bus_routes.read();

                    match TflDataRepository::initialize(should_load_buses).await {
                        Ok(repository) => {
                            log::info_with_category(
                                LogCategory::App,
                                &format!(
                                    "TfL data loaded successfully with {} stations",
                                    repository.stations.len()
                                ),
                            );
                            tfl_data.set(repository);
                        }
                        Err(e) => {
                            log::error_with_category(
                                LogCategory::App,
                                &format!("Failed to load TfL data: {}", e),
                            );
                        }
                    }
                });
            } else {
                logger.info("TfL data already loaded, skipping");
            }
        });
    });

    // Add an effect to reload data when bus routes toggle changes
    use_effect(move || {
        // Skip the initial render
        static mut FIRST_RUN: bool = true;
        let is_first_run = unsafe {
            if FIRST_RUN {
                FIRST_RUN = false;
                true
            } else {
                false
            }
        };

        if !is_first_run {
            with_context("app::reload_tfl_data", LogCategory::App, |logger| {
                let should_load_buses = *load_bus_routes.read();
                logger.info(&format!(
                    "Bus routes toggle changed to {}",
                    should_load_buses
                ));

                // Reload data
                wasm_bindgen_futures::spawn_local(async move {
                    match TflDataRepository::initialize(should_load_buses).await {
                        Ok(repository) => {
                            log::info_with_category(
                                LogCategory::App,
                                &format!(
                                    "TfL data reloaded with bus routes: {}",
                                    should_load_buses
                                ),
                            );
                            tfl_data.set(repository);
                        }
                        Err(e) => {
                            log::error_with_category(
                                LogCategory::App,
                                &format!("Failed to reload TfL data: {}", e),
                            );
                        }
                    }
                });
            });
        }
    });

    // Add an effect to update the map when TFL data is loaded
    use_effect(move || {
        // Only react if the data is loaded
        if tfl_data.read().is_loaded {
            log::info_with_category(LogCategory::App, "TFL data loaded, updating map layers");

            // Update the map with the TFL data
            if let Some(manager) = window()
                .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("mapInstance")).ok())
            {
                let map: crate::maplibre::bindings::Map = manager.clone().into();

                // We need to check if the map style is loaded
                if map.is_style_loaded() {
                    log::info_with_category(
                        LogCategory::App,
                        "Map style loaded, adding TFL data layers",
                    );

                    // Call a helper function to add the TFL data to the map
                    add_tfl_data_to_map(&map, tfl_data.read().clone());
                } else {
                    log::info_with_category(
                        LogCategory::App,
                        "Map style not loaded yet, waiting for 'load' event",
                    );

                    // Create a callback for the 'load' event
                    let tfl_data_clone = tfl_data;
                    let load_callback = Closure::wrap(Box::new(move || {
                        log::info_with_category(
                            LogCategory::App,
                            "Map 'load' event fired, adding TFL data layers",
                        );

                        // If we get here via a callback, we need to get the map again
                        if let Some(window) = window() {
                            if let Ok(map_instance) =
                                js_sys::Reflect::get(&window, &JsValue::from_str("mapInstance"))
                            {
                                let map: crate::maplibre::bindings::Map = map_instance.into();
                                add_tfl_data_to_map(&map, tfl_data_clone.read().clone());
                            }
                        }
                    }) as Box<dyn FnMut()>);

                    // Register the callback
                    map.on("load", &load_callback);

                    // Leak the callback to keep it alive
                    load_callback.forget();
                }
            }

            // Make the TfL data available to JavaScript for the simulation
            let js_code = r#"
                // Create a placeholder for TfL data
                window.__tflData = true; // Simple flag to indicate data is available
            "#;
            let _ = js_sys::eval(js_code);
        }
    });

    // Initialize simulation JS when app loads
    use_effect(move || {
        with_context("app::simulation_init", LogCategory::App, |logger| {
            // Only create the SimulationController if it doesn't already exist
            if let Some(window) = window() {
                if let Ok(simulation_controller) =
                    js_sys::Reflect::get(&window, &JsValue::from_str("SimulationController"))
                {
                    if !simulation_controller.is_undefined() {
                        // SimulationController already exists, no need to add the script
                        // logger.debug("SimulationController already exists, skipping initialization");
                        return;
                    }
                }
            }
            logger.info("Initializing simulation controller script");

            let controller_script = format!(
                r#"
                // Global simulation controller
                const SimulationController = {{
                  initialized: false,
                  running: false,

                  initialize: function() {{
                    console.log("SimulationController.initialize() called");
                    if (this.initialized) {{
                      console.log("Simulation already initialized, skipping");
                      return;
                    }}

                    // Call the Rust initialization function
                    if (typeof window.rust_initialize_simulation === 'function') {{
                      console.log("Calling rust_initialize_simulation()");
                      window.rust_initialize_simulation();
                      this.initialized = true;
                      this.running = true;
                    }} else {{
                      console.error("rust_initialize_simulation function not found");
                    }}
                  }},

                  toggle: function() {{
                    console.log("SimulationController.toggle() called");
                    if (!this.initialized) {{
                      this.initialize();
                      return;
                    }}

                    if (typeof window.rust_toggle_simulation === 'function') {{
                      window.rust_toggle_simulation();
                      this.running = !this.running;
                      console.log("Simulation running:", this.running);
                    }}
                  }},

                  reset: function() {{
                    console.log("SimulationController.reset() called");
                    if (typeof window.rust_reset_simulation === 'function') {{
                      window.rust_reset_simulation();
                      this.running = true;
                      console.log("Simulation reset and running");
                    }}
                  }}
                }};

                // Make it globally available
                window.SimulationController = SimulationController;

                // Only initialize automatically if simulation is enabled
                const simulationEnabled = {0};

                if (simulationEnabled) {{
                  // Initialize when map is ready
                  if (window.mapInstance && window.mapInstance.isStyleLoaded()) {{
                    setTimeout(function() {{
                      SimulationController.initialize();
                    }}, 1000);
                  }} else {{
                    const initInterval = setInterval(function() {{
                      if (window.mapInstance && window.mapInstance.isStyleLoaded()) {{
                        clearInterval(initInterval);
                        setTimeout(function() {{
                          SimulationController.initialize();
                        }}, 1000);
                      }}
                    }}, 1000);
                  }}
                }} else {{
                  console.log("Automatic simulation initialization disabled");
                }}
                "#,
                layers.read().simulation
            );

            if let Err(e) = helpers::add_inline_script(&controller_script) {
                logger.error(&format!("Failed to add simulation script: {:?}", e));
            } else {
                logger.info("Simulation controller script added successfully");
            }
        });
    });

    use_effect(move || {
        with_context("app::simulation_functions", LogCategory::App, |logger| {
            logger.info("Exposing simulation functions");

            // Try to expose simulation functions if available
            match simulation::expose_simulation_functions(Some(tfl_data.read().clone())) {
                Ok(_) => {
                    logger.info("Simulation functions exposed successfully");
                }
                Err(err) => {
                    logger.error(&format!("Failed to expose simulation functions: {:?}", err));
                }
            }

            // Add the controller script
            let controller_script = r#"
    // SimulationController code here...
    "#;

            if let Err(e) = helpers::add_inline_script(controller_script) {
                logger.error(&format!("Failed to add simulation script: {:?}", e));
            } else {
                logger.debug("Additional controller script added");
            }
        })
    });

    // Add an effect to set up the simulation panel connection
    use_effect(move || {
        with_context(
            "app::simulation_panel_connection",
            LogCategory::App,
            |logger| {
                logger.info("Setting up simulation panel connection to JavaScript");

                // Create a clone of the signal for the closure
                let mut show_sim = show_simulation_panel.clone();

                // Create a closure that will open the simulation panel when called from JavaScript
                let open_sim_callback = Closure::wrap(Box::new(move || {
                    log::info_with_category(
                        LogCategory::App,
                        "openTflSimulationPanel called from JavaScript",
                    );
                    show_sim.set(true);
                }) as Box<dyn FnMut()>);

                // Expose the closure to JavaScript
                if let Some(window) = window() {
                    if let Err(e) = js_sys::Reflect::set(
                        &window,
                        &JsValue::from_str("openTflSimulationPanel"),
                        open_sim_callback.as_ref(),
                    ) {
                        logger.error(&format!("Failed to set openTflSimulationPanel: {:?}", e));
                    } else {
                        logger.info("Successfully exposed openTflSimulationPanel to JavaScript");
                    }
                }

                // Forget the closure to prevent memory leaks
                open_sim_callback.forget();
            },
        );
    });

    // Add an effect to update the simulation vehicle count
    use_effect(move || {
        let timer = std::time::Duration::from_secs(1);

        let mut update_vehicle_count = move || {
            if *show_simulation_panel.read() {
                // Get the vehicle count from the simulation state
                let count = simulation::with_simulation_state_ref(|state| state.vehicles.len());
                vehicle_count.set(Some(count));
            }
        };

        // Set up an interval to update the vehicle count
        let mut interval_handle = None;
        if let Some(window) = window() {
            let callback =
                Closure::wrap(Box::new(update_vehicle_count.clone()) as Box<dyn FnMut()>);
            if let Ok(handle) = window.set_interval_with_callback_and_timeout_and_arguments(
                callback.as_ref().unchecked_ref(),
                1000,
                &js_sys::Array::new(),
            ) {
                interval_handle = Some(handle);
            }
            callback.forget();
        }

        // Run the function once immediately
        update_vehicle_count();

        // Return cleanup function
        (move || {
            if let Some(handle) = interval_handle {
                if let Some(window) = window() {
                    window.clear_interval_with_handle(handle);
                }
            }
        })()
    });

    // Add an effect to connect the key panel (glorified onclick event handler)
    use_effect(move || {
        with_context("app::key_panel_connection", LogCategory::App, |logger| {
            logger.info("Setting up key panel connection to JavaScript");

            // Create a clone of the signal for the closure
            let mut show_key = show_key_panel;

            // Create a closure that will open the key panel when called from JavaScript
            // Don't capture logger in this closure!
            let open_key_callback = Closure::wrap(Box::new(move || {
                // Use direct log calls instead of the captured logger
                log::debug_with_category(
                    LogCategory::App,
                    "openTflKeyPanel called from JavaScript",
                );
                show_key.set(true);
            }) as Box<dyn FnMut()>);

            // Expose the closure to JavaScript
            if let Some(window) = window() {
                if let Err(e) = js_sys::Reflect::set(
                    &window,
                    &JsValue::from_str("openTflKeyPanel"),
                    open_key_callback.as_ref(),
                ) {
                    logger.error(&format!("Failed to set openTflKeyPanel: {:?}", e));
                } else {
                    logger.info("Successfully exposed openTflKeyPanel to JavaScript");
                }
            }

            // Forget the closure to prevent memory leaks
            open_key_callback.forget();
        });
    });

    // Add an effect to set up the layer panel connection
    use_effect(move || {
        with_context("app::layer_panel_connection", LogCategory::App, |logger| {
            logger.info("Setting up layer panel connection to JavaScript");

            // Create a clone of the signal for the closure
            let mut show_layers = show_layers_panel.clone();

            // Create a closure that will open the layer panel when called from JavaScript
            let open_layer_panel_callback = Closure::wrap(Box::new(move || {
                log::info_with_category(
                    LogCategory::App,
                    "openTflLayerPanel called from JavaScript",
                );
                show_layers.set(true);
            }) as Box<dyn FnMut()>);

            // Expose the closure to JavaScript
            if let Some(window) = window() {
                if let Err(e) = js_sys::Reflect::set(
                    &window,
                    &JsValue::from_str("openTflLayerPanel"),
                    open_layer_panel_callback.as_ref(),
                ) {
                    logger.error(&format!("Failed to set openTflLayerPanel: {:?}", e));
                } else {
                    logger.info("Successfully exposed openTflLayerPanel to JavaScript");
                }
            }

            // Forget the closure to prevent memory leaks
            open_layer_panel_callback.forget();
        });
    });

    rsx! {
        LineCss {}

        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TFL_CSS }
        document::Link { rel: "stylesheet", href: KEY_CSS }
        document::Link { rel: "stylesheet", href: SIM_CSS }
        document::Link { rel: "stylesheet", href: LAYER_CSS }

        header {
            img { src: LOGO_SVG }
            p { "Real-time TfL network simulation" }

            nav {
                ul {
                    li { a { href: "#", "About" } }
                    li { a { href: "#", "Stats" } }
                    li { a { href: "#", "Exports" } }
                }
            }
        }

        main {
            class: "app-content",

            // Main map container
            Canvas { layers: layers, tfl_data: tfl_data }

            // Layer panel component - conditionally shown
            LayerPanel {
                visible: *show_layers_panel.read(),
                layers: layers,
                load_bus_routes: load_bus_routes,
                on_close: move |_| show_layers_panel.set(false)
            }

            // Key panel component - conditionally shown
            KeyPanel {
                visible: *show_key_panel.read(),
                on_close: move |_| show_key_panel.set(false)
            }

            SimulationPanel {
                visible: *show_simulation_panel.read(),
                is_paused: *simulation_is_paused.read(),
                vehicle_count: *vehicle_count.read(),
                on_close: move |_| show_simulation_panel.set(false),
                on_toggle: move |_| {
                    // Check if simulation has been initialized
                    if !*simulation_initialized.read() {
                        // If not initialized, initialize it
                        simulation::initialize_simulation(Some(tfl_data.read().clone()));
                        simulation_initialized.set(true);
                        simulation_is_paused.set(false); // Start running
                    } else {
                        // Otherwise just toggle pause state
                        simulation::toggle_simulation();
                        let is_currently_paused = *simulation_is_paused.read();
                        simulation_is_paused.set(!is_currently_paused);
                    }
                },
                on_reset: move |_| {
                    // Reset always initializes
                    simulation::reset_simulation(Some(tfl_data.read().clone()));
                    simulation_initialized.set(true);
                    simulation_is_paused.set(false);
                }
            }
        }
    }
}

/// Helper function to add TFL data layers to an already initialized map
fn add_tfl_data_to_map(map: &crate::maplibre::bindings::Map, tfl_data: TflDataRepository) {
    with_context("add_tfl_data_to_map", LogCategory::Map, |logger| {
        logger.info("Adding TFL data layers to map");

        // Add route geometries from our new data
        if let Ok(route_data) = crate::data::generate_all_route_geometries(&tfl_data) {
            logger.info(&format!(
                "Adding {} TFL route geometries to map",
                route_data.len()
            ));

            for (line_id, route_geojson) in route_data {
                logger.debug(&format!("Adding {} route geometry", line_id));
                web_sys::console::log_1(&route_geojson);
                let source_id = format!("{}-route", line_id);
                let layer_id = format!("{}-route-layer", line_id);

                // Make sure the layer doesn't already exist
                if map.get_layer(&layer_id).is_none() {
                    // Add the source
                    map.add_source(&source_id, &route_geojson);

                    // Get the appropriate color for this line
                    let color = get_line_color(&line_id);

                    let route_mode = tfl_data
                        .routes
                        .get(&line_id)
                        .and_then(|directions| directions.values().next())
                        .and_then(|response| response.first())
                        .map(|route_sequence| route_sequence.mode.to_lowercase())
                        .unwrap_or_else(|| "train".to_string());
                    let width = if route_mode == "bus" { 0.0 } else { 3.0 };

                    // Add the layer with a dashed style to distinguish from simplified line data
                    if let Ok(route_layer) = create_line_layer(&layer_id, &source_id, &color, width)
                    {
                        map.add_layer(&route_layer);
                        logger.debug(&format!("Added {} route geometry", line_id));
                    }
                } else {
                    logger.debug(&format!("{} route layer already exists, skipping", line_id));
                }
            }
        } else {
            logger.error("Failed to generate route geometries");
        }

        // Add all stations as a GeoJSON source
        if let Ok(stations_geojson) = crate::data::stations_to_geojson(&tfl_data.stations) {
            // Make sure the source doesn't already exist
            if map.get_layer("tfl-stations-layer").is_none() {
                logger.info(&format!(
                    "Adding {} stations to map",
                    tfl_data.stations.len()
                ));

                // Add the source
                web_sys::console::log_1(&stations_geojson);
                map.add_source("tfl-stations", &stations_geojson);

                // Add a circle layer for the stations
                if let Ok(stations_layer) =
                    create_circle_layer("tfl-stations-layer", "tfl-stations")
                {
                    map.add_layer(&stations_layer);
                    logger.debug("Added stations layer");
                }

                // Add a label layer for the stations
                if let Ok(labels_layer) = create_label_layer("tfl-station-labels", "tfl-stations") {
                    map.add_layer(&labels_layer);
                    logger.debug("Added station labels layer");
                }
            } else {
                logger.debug("Stations layer already exists, skipping");
            }
        } else {
            logger.error("Failed to convert stations to GeoJSON");
        }

        // Commented out as this is deprecated: uncomment to see new lines before adding their routes
        // // Add all tube lines (NB this is being incrementally deprecated)
        // if let Ok(line_data) = crate::data::generate_all_line_data(&tfl_data) {
        //     logger.info(&format!("Adding {} TFL lines to map", line_data.len()));

        //     for (line_name, line_geojson, color) in line_data {
        //         // Skip lines that have proper route data
        //         match line_name.as_str() {
        //             "bakerloo" | "central" | "circle" | "district" | "hammersmith-city"
        //             | "jubilee" | "metropolitan" | "northern" | "piccadilly" | "victoria"
        //             | "waterloo-city" | "elizabeth" | "thameslink" | "tram" | "dlr" | "london-cable-car" => {
        //                 logger.debug(&format!(
        //                     "Skipping {} line - using route data instead",
        //                     line_name
        //                 ));
        //                 continue; // Skip this iteration
        //             }
        //             _ => {} // Process other lines normally
        //         }
        //         web_sys::console::log_1(&line_geojson);
        //         let source_id = format!("{}-line", line_name);
        //         let layer_id = format!("{}-line-layer", line_name);

        //         // Make sure the layer doesn't already exist
        //         if map.get_layer(&layer_id).is_none() {
        //             // Add the source
        //             map.add_source(&source_id, &line_geojson);

        //             // Add the layer
        //             if let Ok(line_layer) = create_line_layer(&layer_id, &source_id, &color, 4.0) {
        //                 map.add_layer(&line_layer);
        //                 // Anything set this way is invisible (so as to deprecate as we migratet to routes)
        //                 map.set_layout_property(
        //                     &layer_id,
        //                     "visibility",
        //                     &JsValue::from_str("none"),
        //                 );
        //                 logger.debug(&format!("Added {} line", line_name));
        //             }
        //         } else {
        //             logger.debug(&format!("{} line already exists, skipping", line_name));
        //         }
        //     }
        // } else {
        //     logger.error("Failed to generate line data");
        // }

        logger.info("TFL data layers added to map");
    });
}
