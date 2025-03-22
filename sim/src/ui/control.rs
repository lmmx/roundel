// src/ui/control.rs

use crate::model::{GLOBAL_STATE, VehicleType};
use std::cell::RefCell;

#[derive(Default)]
pub struct SimulationControl {
    pub paused: bool,
    pub update_interval_ms: u32,
    pub auto_adjust: bool,
    pub frame_count: u32,
    pub total_frame_time: f64,
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
        update_interval_ms: 16, // Default to ~60fps
        auto_adjust: true,
        frame_count: 0,
        total_frame_time: 0.0,
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

    /// Auto-adjust the update interval based on performance
    pub fn auto_adjust_interval(&mut self, frame_time_ms: f64) {
        // Only consider the first 10 frames for auto-adjustment
        if self.frame_count < 10 {
            self.total_frame_time += frame_time_ms;
            self.frame_count += 1;

            // After collecting 10 frames of data, make a decision
            if self.frame_count == 10 {
                let avg_frame_time = self.total_frame_time / 10.0;

                // If average frame time is > 8ms (allowing some headroom),
                // reduce to 30fps (33ms interval) instead of 60fps (16ms interval)
                if avg_frame_time > 8.0 {
                    self.update_interval_ms = 33; // ~30fps
                } else {
                    self.update_interval_ms = 16; // ~60fps
                }

                web_sys::console::log_1(
                    &format!(
                        "Auto-adjusted to {}ms interval (avg frame time: {:.2}ms)",
                        self.update_interval_ms, avg_frame_time
                    )
                    .into(),
                );

                // Disable auto-adjust after initial setting
                self.auto_adjust = false;
            }
        }
    }
}
