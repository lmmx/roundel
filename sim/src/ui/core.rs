// src/ui/core.rs

use super::control::SIMULATION_CONTROL;
use super::draw::{draw_routes, draw_stats, draw_vehicles};
use super::input::{attach_control_listeners, attach_mouse_listeners, attach_wheel_listener};
use crate::model::GLOBAL_STATE;
use crate::ui::camera::CAMERA;
use crate::ui::input::attach_vehicle_selection_listener;
use js_sys::Date;
use once_cell::sync::OnceCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, console};

// Use OnceCell to store our interval ID so we can clear and reset it
static ANIMATION_INTERVAL_ID: OnceCell<i32> = OnceCell::new();

/// Called once when the Wasm module loads:
/// 1) Read the checkbox state to determine initial mode
/// 2) Initialize camera from DOM values
/// 3) Initialize routes/vehicles based on the initial mode
/// 4) Start the update loop (with adaptive interval)
/// 5) Attach mouse events for panning and wheel event for zoom
/// 6) Attach control listeners for simulation controls
/// 7) Load real data from TSV files if needed
/// 8) Attach a data source switch listener
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    // Read the initial checkbox state from the DOM
    let use_random_routes = read_initial_checkbox_state()?;
    console::log_1(
        &format!(
            "Initial mode: {}",
            if use_random_routes {
                "random routes"
            } else {
                "real data"
            }
        )
        .into(),
    );

    // Initialize camera from DOM values
    super::camera::initialize_camera_from_dom();

    GLOBAL_STATE.with(|cell| {
        let mut state = cell.borrow_mut();

        // Set our global data source mode based on checkbox
        state.set_debug_mode(use_random_routes);

        // If using random routes, build them immediately
        if use_random_routes {
            state.build_random_routes();
        }

        // Now initialize vehicles
        state.init_vehicles();
    });

    // Initialize vehicle counts
    SIMULATION_CONTROL.with(|cell| {
        cell.borrow_mut().update_vehicle_counts();
    });

    // 2) Repeated update & draw
    start_animation_loop()?;

    // 3) Attach mouse & wheel listeners
    attach_mouse_listeners()?;
    attach_wheel_listener()?;

    // 4) Attach simulation control listeners
    attach_control_listeners()?;

    // 5) Attach vehicle following listeners
    attach_vehicle_selection_listener()?;

    // 6) Load real TSV data and update routes (only if not using random routes)
    if !use_random_routes {
        load_real_route_data();
    }

    // 7) Tie the data source checkbox to set_debug_mode
    attach_data_source_listener()?;

    Ok(())
}

/// Reads the initial state of the checkbox from the DOM
fn read_initial_checkbox_state() -> Result<bool, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("No document object"))?;

    // Get the checkbox element
    if let Some(checkbox_el) = document.get_element_by_id("debugModeCheckbox") {
        // Cast it to HtmlInputElement
        if let Ok(checkbox) = checkbox_el.dyn_into::<web_sys::HtmlInputElement>() {
            // Return its checked state
            return Ok(checkbox.checked());
        }
    }

    // If we can't find the checkbox or cast it, default to false (real data mode)
    console::log_1(&"Could not find checkbox, defaulting to real data mode".into());
    Ok(false)
}

/// Asynchronously load TSV files and update routes with real data
fn load_real_route_data() {
    console::log_1(&"Starting to load real route data...".into());

    // Use wasm_bindgen_futures to spawn an async task
    spawn_local(async {
        match load_tsv_files().await {
            Ok((bus_data, tube_data)) => {
                console::log_1(&"Successfully loaded TSV files, updating routes".into());

                // Only update routes if we're still in real data mode
                GLOBAL_STATE.with(|cell| {
                    let mut state = cell.borrow_mut();
                    if !state.debug_mode {
                        state.update_with_real_routes(&bus_data, &tube_data);
                        console::log_1(&"Routes updated with real data".into());
                    } else {
                        console::log_1(&"Random routes mode active, ignoring loaded data".into());
                    }
                });

                // Update vehicle counts after the change
                SIMULATION_CONTROL.with(|cell| {
                    let mut control = cell.borrow_mut();
                    control.update_vehicle_counts();

                    // Resume simulation now that data is loaded (if we're in real data mode)
                    GLOBAL_STATE.with(|cell| {
                        let state = cell.borrow();
                        if !state.debug_mode {
                            control.paused = false;
                            console::log_1(&"Resuming simulation with real data".into());
                        }
                    });
                });
            }
            Err(e) => {
                console::log_1(&format!("Error loading TSV files: {:?}", e).into());
                console::log_1(&"Continuing without real data.".into());

                // If we failed to load data but are in real data mode,
                // resume the simulation anyway to prevent it staying frozen
                SIMULATION_CONTROL.with(|cell| {
                    let mut control = cell.borrow_mut();
                    control.paused = false;
                });
            }
        }
    });
}

/// Fetch both TSV files
async fn load_tsv_files() -> Result<(String, String), JsValue> {
    use crate::model::route_builder::fetch_tsv_file;

    // Fetch both files in parallel
    let bus_data_future = fetch_tsv_file("bus_routes.tsv");
    let tube_data_future = fetch_tsv_file("tube_routes.tsv");

    // Await both futures
    let bus_data = bus_data_future.await?;
    let tube_data = tube_data_future.await?;

    Ok((bus_data, tube_data))
}

/// Creates a closure that runs repeatedly and updates + draws.
fn start_animation_loop() -> Result<(), JsValue> {
    let closure = Closure::wrap(Box::new(move || {
        let t_start = Date::now();

        // Get pause state
        let paused = SIMULATION_CONTROL.with(|cell| cell.borrow().paused);

        // 1) Update vehicles (if not paused)
        if !paused {
            GLOBAL_STATE.with(|cell| {
                cell.borrow_mut().update_all();
            });

            // Update vehicle counts
            SIMULATION_CONTROL.with(|cell| {
                cell.borrow_mut().update_vehicle_counts();
            });

            // Check if we need to update camera for follow mode
            CAMERA.with(|c| {
                let mut cam = c.borrow_mut();
                if cam.follow_mode && cam.selected_vehicle_index.is_some() {
                    let index = cam.selected_vehicle_index.unwrap();
                    GLOBAL_STATE.with(|cell| {
                        let state = cell.borrow();
                        if index < state.vehicles.len() {
                            let vehicle = &state.vehicles[index];
                            // Get canvas dimensions
                            if let Some(win) = window() {
                                if let Some(document) = win.document() {
                                    if let Some(canvas_el) = document.get_element_by_id("myCanvas")
                                    {
                                        if let Ok(canvas) =
                                            canvas_el.dyn_into::<HtmlCanvasElement>()
                                        {
                                            let canvas_width = canvas.width() as f32;
                                            let canvas_height = canvas.height() as f32;

                                            // Center camera on selected vehicle
                                            cam.pan_x =
                                                vehicle.x - (canvas_width / cam.scale / 2.0);
                                            cam.pan_y =
                                                vehicle.y - (canvas_height / cam.scale / 2.0);
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            });
        }

        // 2) Draw everything (even if paused, to show current state)
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(canvas_el) = document.get_element_by_id("myCanvas") {
                    if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                        if let Ok(ctx) = canvas
                            .get_context("2d")
                            .unwrap()
                            .unwrap()
                            .dyn_into::<CanvasRenderingContext2d>()
                        {
                            let w = canvas.width() as f64;
                            let h = canvas.height() as f64;
                            ctx.clear_rect(0.0, 0.0, w, h);

                            draw_routes(&ctx);
                            draw_vehicles(&ctx);
                            draw_stats(&ctx);
                        }
                    }
                }
            }
        }

        let t_end = Date::now();
        let frame_time_ms = t_end - t_start;

        // Log performance
        console::log_1(&format!("Update & draw took {:.3} ms", frame_time_ms).into());
    }) as Box<dyn FnMut()>);

    // Set the initial interval
    let update_interval = SIMULATION_CONTROL.with(|cell| cell.borrow().update_interval_ms);

    let window = web_sys::window().unwrap();
    let interval_id = window.set_interval_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        update_interval as i32,
    )?;

    // Store the interval ID so we can update it later
    let _ = ANIMATION_INTERVAL_ID.set(interval_id);

    // Keep the closure alive
    closure.forget();

    Ok(())
}

/// Change the animation interval to a new value
#[wasm_bindgen]
pub fn change_animation_interval(new_interval_ms: u32) -> Result<(), JsValue> {
    // Update the stored interval value
    SIMULATION_CONTROL.with(|cell| {
        cell.borrow_mut().update_interval_ms = new_interval_ms;
    });

    // Get the old interval ID
    if let Some(old_interval_id) = ANIMATION_INTERVAL_ID.get() {
        // Clear the old interval
        let window = web_sys::window().unwrap();
        window.clear_interval_with_handle(*old_interval_id);

        // Start a new animation loop
        start_animation_loop()?;

        console::log_1(&format!("Changed animation interval to {}ms", new_interval_ms).into());
    }

    Ok(())
}

/// Toggle the pause state of the simulation
#[wasm_bindgen]
pub fn toggle_pause() -> bool {
    SIMULATION_CONTROL.with(|cell| {
        let mut control = cell.borrow_mut();
        control.paused = !control.paused;
        console::log_1(&format!("Simulation paused: {}", control.paused).into());
        control.paused
    })
}

/// Listens for changes to the "debugModeCheckbox" for data source selection
fn attach_data_source_listener() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window object")?;
    let document = window.document().ok_or("No document object")?;
    let checkbox_el = document
        .get_element_by_id("debugModeCheckbox")
        .ok_or("Could not find element #debugModeCheckbox")?;

    // Cast it to HtmlInputElement so we can read `.checked`
    let checkbox: web_sys::HtmlInputElement = checkbox_el
        .dyn_into()
        .map_err(|_| "Element is not an HtmlInputElement")?;

    // Create a closure that fires on "change"
    let cb_clone = checkbox.clone();
    let closure = Closure::wrap(Box::new(move || {
        let is_checked = cb_clone.checked();
        set_data_source_mode(is_checked);
    }) as Box<dyn FnMut()>);

    checkbox.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
    closure.forget(); // keep closure alive

    Ok(())
}

/// Function to switch between random routes and real data routes
#[wasm_bindgen]
pub fn set_data_source_mode(use_random: bool) {
    // First, pause the simulation to prevent updates while we're changing data
    let was_paused = SIMULATION_CONTROL.with(|cell| {
        let mut control = cell.borrow_mut();
        let was_paused = control.paused;
        control.paused = true;
        was_paused
    });

    GLOBAL_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.set_debug_mode(use_random);

        // Clear BOTH existing routes AND vehicles to prevent any stale references
        state.routes.clear();
        state.vehicles.clear();

        if use_random {
            // If using random routes, build them immediately
            console::log_1(&"Building random routes...".into());
            state.build_random_routes();
            // Now initialize vehicles for these new routes
            state.init_vehicles();
            console::log_1(&"Switched to random routes mode".into());
        } else {
            // If using real data, we'll start loading it
            // Keep the simulation paused until data loads
            console::log_1(&"Switched to real data mode, loading routes...".into());
            drop(state); // Release the borrow before the async operation
            load_real_route_data();
        }
    });

    // Update vehicle counts
    SIMULATION_CONTROL.with(|cell| {
        let mut control = cell.borrow_mut();
        control.update_vehicle_counts();

        // Only restore previous pause state if we were using random routes
        // (for real data, we'll keep it paused until loaded)
        if use_random && !was_paused {
            control.paused = false;
        }
    });
}

/// Legacy function to maintain compatibility, delegates to set_data_source_mode
#[wasm_bindgen]
pub fn set_debug_mode(enable: bool) {
    set_data_source_mode(enable);
    console::log_1(&format!("debug_mode set to {}", enable).into());
}
