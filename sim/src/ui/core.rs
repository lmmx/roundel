// src/ui/core.rs

use super::control::SIMULATION_CONTROL;
use super::draw::{draw_routes, draw_stats, draw_vehicles};
use super::input::{attach_control_listeners, attach_mouse_listeners, attach_wheel_listener};
use crate::model::GLOBAL_STATE;
use js_sys::Date;
use once_cell::sync::OnceCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};
use wasm_bindgen_futures::spawn_local;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, console};

// Use OnceCell to store our interval ID so we can clear and reset it
static ANIMATION_INTERVAL_ID: OnceCell<i32> = OnceCell::new();

/// Called once when the Wasm module loads:
/// 1) Initialize routes/vehicles
/// 2) Start the update loop (with adaptive interval)
/// 3) Attach mouse events for panning and wheel event for zoom
/// 4) Attach control listeners for simulation controls
/// 5) Load real data from TSV files
/// 6) Attach a debug-mode checkbox listener
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    // Decide whether or not to allow random fallback initially
    let debug_mode = false;

    GLOBAL_STATE.with(|cell| {
        let mut state = cell.borrow_mut();

        // Set our global debug mode
        state.set_debug_mode(debug_mode);

        // Now attempt to initialize routes & vehicles
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

    // 5) Load real TSV data and update routes
    load_real_route_data();

    // 6) Tie the debug checkbox to set_debug_mode
    attach_debug_checkbox_listener()?;

    Ok(())
}

/// Asynchronously load TSV files and update routes with real data
fn load_real_route_data() {
    console::log_1(&"Starting to load real route data...".into());

    // Use wasm_bindgen_futures to spawn an async task
    spawn_local(async {
        match load_tsv_files().await {
            Ok((bus_data, tube_data)) => {
                console::log_1(&"Successfully loaded TSV files, updating routes".into());

                // Update routes with real data
                GLOBAL_STATE.with(|cell| {
                    let mut state = cell.borrow_mut();
                    state.update_with_real_routes(&bus_data, &tube_data);
                });

                // Update vehicle counts after the change
                SIMULATION_CONTROL.with(|cell| {
                    cell.borrow_mut().update_vehicle_counts();
                });
            }
            Err(e) => {
                console::log_1(&format!("Error loading TSV files: {:?}", e).into());
                console::log_1(&"Continuing without real data.".into());
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

/// Listens for changes to the "debugModeCheckbox" in the top-right
fn attach_debug_checkbox_listener() -> Result<(), JsValue> {
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
        set_debug_mode(is_checked);
    }) as Box<dyn FnMut()>);

    checkbox.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
    closure.forget(); // keep closure alive

    Ok(())
}

/// Exported function to set debug mode at runtime (also called by the checkbox)
#[wasm_bindgen]
pub fn set_debug_mode(enable: bool) {
    GLOBAL_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.set_debug_mode(enable);
    });
    console::log_1(&format!("debug_mode set to {}", enable).into());
}
