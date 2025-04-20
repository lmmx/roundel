use super::model::{Route, Vehicle};
use crate::utils::log::{LogCategory, with_context};
use std::cell::RefCell;

/// Core shared state that contains all vehicles and simulation data
#[derive(Default)]
pub struct SimulationState {
    pub vehicles: Vec<Vehicle>,
    pub routes: Vec<Route>,
    pub is_paused: bool,
    pub animation_frame_id: Option<i32>,
}

// We'll use thread_local for our global state
thread_local! {
    static SIMULATION_STATE: RefCell<SimulationState> = RefCell::new(SimulationState::default());
}

/// Initialize the simulation state with vehicles and routes
pub fn initialize_state(routes: Vec<Route>, vehicles: Vec<Vehicle>) {
    with_context("initialize_state", LogCategory::Simulation, |logger| {
        logger.info(&format!(
            "Initializing simulation state with {} routes and {} vehicles",
            routes.len(),
            vehicles.len()
        ));

        // Store in global state
        SIMULATION_STATE.with(|state| {
            let mut sim_state = state.borrow_mut();
            sim_state.routes = routes;
            sim_state.vehicles = vehicles;
            sim_state.is_paused = false;
            sim_state.animation_frame_id = None;
        });
    });
}

/// Set the animation frame ID in the state
pub fn set_animation_frame_id(id: i32) {
    SIMULATION_STATE.with(|state| {
        let mut sim_state = state.borrow_mut();
        sim_state.animation_frame_id = Some(id);
    });
}

/// Toggle the simulation pause state and return the new state
pub fn toggle_pause() -> bool {
    SIMULATION_STATE.with(|state| {
        let mut sim_state = state.borrow_mut();
        sim_state.is_paused = !sim_state.is_paused;
        sim_state.is_paused
    })
}

pub fn is_paused() -> bool {
    SIMULATION_STATE.with(|state| {
        let sim_state = state.borrow();
        sim_state.is_paused
    })
}

/// Get a reference to the current animation frame ID if it exists
pub fn get_animation_frame_id() -> Option<i32> {
    SIMULATION_STATE.with(|state| {
        let sim_state = state.borrow();
        sim_state.animation_frame_id
    })
}

/// Execute an action with the simulation state
pub fn with_simulation_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut SimulationState) -> R,
{
    SIMULATION_STATE.with(|state| {
        let mut sim_state = state.borrow_mut();
        f(&mut sim_state)
    })
}

/// Execute an action with an immutable reference to the simulation state
pub fn with_simulation_state_ref<F, R>(f: F) -> R
where
    F: FnOnce(&SimulationState) -> R,
{
    SIMULATION_STATE.with(|state| {
        let sim_state = state.borrow();
        f(&sim_state)
    })
}

/// Get the vehicle count
pub fn get_vehicle_count() -> usize {
    with_simulation_state_ref(|state| state.vehicles.len())
}

/// Debug function to log important simulation state
pub fn debug_simulation_state() {
    // Only log periodically to avoid console spam
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        if COUNTER % 60 != 0 {
            // Log every ~60 frames (roughly 1 second)
            return;
        }
    }

    with_simulation_state_ref(|sim_state| {
        with_context(
            "debug_simulation_state",
            LogCategory::Simulation,
            |logger| {
                // Log general state
                logger.debug(&format!(
                    "Simulation state: {} vehicles, paused: {}",
                    sim_state.vehicles.len(),
                    sim_state.is_paused
                ));

                // Log a sample vehicle
                if !sim_state.vehicles.is_empty() {
                    let sample = &sim_state.vehicles[0];
                    logger.debug(&format!(
                        "Sample vehicle: id={}, type={:?}, pos=({:.4}, {:.4})",
                        sample.id, sample.vehicle_type, sample.lng, sample.lat
                    ));
                }

                // Check if vehicles source exists and log its state
                let js_code = r#"
            let result = "unknown";
            if (window.mapInstance) {
                const source = window.mapInstance.getSource('vehicles-source');
                if (source) {
                    try {
                        const data = source._data;
                        const features = data.features || [];
                        result = `Source exists, ${features.length} features`;
                    } catch (e) {
                        result = `Source exists but error: ${e.message}`;
                    }
                } else {
                    result = "Source does not exist";
                }
            } else {
                result = "Map instance not found";
            }
            result;
            "#;

                match js_sys::eval(js_code) {
                    Ok(result) => {
                        if let Some(result_str) = result.as_string() {
                            logger.debug(&format!("Vehicles source check: {}", result_str));
                        }
                    }
                    Err(err) => {
                        logger.error(&format!("Failed to check vehicles source: {:?}", err));
                    }
                }
            },
        )
    });
}
