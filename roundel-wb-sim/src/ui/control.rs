// src/ui/control.rs

use crate::model::{GLOBAL_STATE, VehicleType};
use std::cell::RefCell;

#[derive(Default)]
pub struct SimulationControl {
    pub paused: bool,
    pub update_interval_ms: u32,
    pub vehicle_counts: VehicleCounts,
}

/// Stores counts of vehicles by type
#[derive(Default, Clone, Copy)]
pub struct VehicleCounts {
    pub buses: usize,
    pub trains: usize,
    pub total: usize,
}

thread_local! {
    pub static SIMULATION_CONTROL: RefCell<SimulationControl> = RefCell::new(SimulationControl {
        paused: false,
        update_interval_ms: 33, // Default to ~30fps
        vehicle_counts: VehicleCounts::default(),
    });
}

/// Get current vehicle counts
pub fn get_vehicle_counts() -> VehicleCounts {
    SIMULATION_CONTROL.with(|cell| cell.borrow().vehicle_counts)
}

impl SimulationControl {
    /// Update vehicle counts by type
    pub fn update_vehicle_counts(&mut self) {
        let mut counts = VehicleCounts::default();

        GLOBAL_STATE.with(|cell| {
            let state = cell.borrow();
            for vehicle in &state.vehicles {
                match vehicle.vehicle_type {
                    VehicleType::Bus => counts.buses += 1,
                    VehicleType::Train => counts.trains += 1,
                }
            }
        });

        counts.total = counts.buses + counts.trains;
        self.vehicle_counts = counts;
    }
}
