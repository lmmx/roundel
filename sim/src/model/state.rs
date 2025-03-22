// src/model/state.rs

use crate::model::route_builder::{
    build_random_bus_routes, build_random_train_routes, build_real_bus_routes,
    build_real_train_routes,
};
use crate::model::{Route, Vehicle, VehicleType};
use std::cell::RefCell;

/// Holds all simulation data, including a global "debug_mode" toggle
pub struct SharedState {
    pub routes: Vec<Route>,
    pub vehicles: Vec<Vehicle>,

    /// Toggle whether to generate random fallback routes when none are loaded
    pub debug_mode: bool,
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            vehicles: Vec::new(),
            debug_mode: false, // default off
        }
    }

    /// Enable or disable debug mode at runtime
    pub fn set_debug_mode(&mut self, debug: bool) {
        self.debug_mode = debug;
    }

    /// Builds random routes for both trains and buses
    pub fn build_random_routes(&mut self) {
        // fetch random train routes
        let mut trains = build_random_train_routes();
        // fetch random bus routes
        let mut buses = build_random_bus_routes();

        // combine them
        self.routes.clear();
        self.routes.append(&mut trains); // first 10 => trains
        self.routes.append(&mut buses); // next 100 => buses
    }

    /// Updates routes with real data from TSV files
    pub fn update_with_real_routes(&mut self, bus_tsv: &str, train_tsv: &str) {
        web_sys::console::log_1(&"Updating routes with real data...".into());

        // Build routes from real TSV data
        let mut trains = build_real_train_routes(train_tsv);
        let mut buses = build_real_bus_routes(bus_tsv);

        // combine them
        self.routes.clear();
        self.routes.append(&mut trains); // first ~11 => trains
        self.routes.append(&mut buses); // next ~125 => buses

        // Recreate vehicles with the new routes
        self.vehicles.clear();
        self.init_vehicles();

        web_sys::console::log_1(
            &format!(
                "Routes updated: {} train routes, {} bus routes",
                trains.len(),
                buses.len()
            )
            .into(),
        );
    }

    /// Create the vehicles: 1,000 buses + 1,000 trains
    pub fn init_vehicles(&mut self) {
        // If we have no routes, only build random ones if debug_mode is true.
        if self.routes.is_empty() {
            if self.debug_mode {
                web_sys::console::log_1(&"Debug mode on: building random routes...".into());
                self.build_random_routes();
            } else {
                web_sys::console::log_1(
                    &"No routes loaded (and debug mode is off). Doing nothing.".into(),
                );
                return;
            }
        }

        let rng = || js_sys::Math::random() as f32;

        // Determine how many train routes we have (should be at least 10)
        let train_routes = self.routes.iter().take(11).count().min(10);

        // Buses => route indices after train routes
        let bus_start_index = train_routes;
        let num_bus_routes = self.routes.len() - train_routes;
        let vehicles_per_bus_route = if num_bus_routes > 0 {
            1000 / num_bus_routes
        } else {
            10
        };

        web_sys::console::log_1(
            &format!(
                "Creating {} buses across {} routes",
                vehicles_per_bus_route * num_bus_routes,
                num_bus_routes
            )
            .into(),
        );

        // Create buses
        for r_i in 0..num_bus_routes {
            let route_index = bus_start_index + r_i;
            if route_index >= self.routes.len() {
                continue;
            }

            let route = &self.routes[route_index];
            let count_fwd = vehicles_per_bus_route / 2;
            let count_bwd = vehicles_per_bus_route - count_fwd;

            // forward
            for i in 0..count_fwd {
                if route.stations.len() < 2 {
                    continue;
                }
                let last_station = 0;
                let next_station = 1;
                let f_lo = (i as f32) / (count_fwd as f32);
                let f_hi = ((i + 1) as f32) / (count_fwd as f32);
                let fraction = f_lo + (f_hi - f_lo) * rng();
                let speed = 0.005 + rng() * 0.01;
                let mut v = Vehicle {
                    vehicle_type: VehicleType::Bus,
                    route_index,
                    direction: 1,
                    last_station,
                    next_station,
                    fraction,
                    speed,
                    x: 0.0,
                    y: 0.0,
                };
                v.update_position(route);
                self.vehicles.push(v);
            }
            // backward
            for i in 0..count_bwd {
                let n_stations = route.stations.len();
                if n_stations < 2 {
                    continue;
                }
                let last_station = n_stations - 1;
                let next_station = n_stations - 2;
                let f_lo = (i as f32) / (count_bwd as f32);
                let f_hi = ((i + 1) as f32) / (count_bwd as f32);
                let fraction = f_lo + (f_hi - f_lo) * rng();
                let speed = 0.005 + rng() * 0.01;
                let mut v = Vehicle {
                    vehicle_type: VehicleType::Bus,
                    route_index,
                    direction: -1,
                    last_station,
                    next_station,
                    fraction,
                    speed,
                    x: 0.0,
                    y: 0.0,
                };
                v.update_position(route);
                self.vehicles.push(v);
            }
        }

        // Trains => route indices 0..train_routes
        let vehicles_per_train_route = if train_routes > 0 {
            1000 / train_routes
        } else {
            100
        };

        web_sys::console::log_1(
            &format!(
                "Creating {} trains across {} routes",
                vehicles_per_train_route * train_routes,
                train_routes
            )
            .into(),
        );

        for r_i in 0..train_routes {
            let route_index = r_i;
            if route_index >= self.routes.len() {
                continue;
            }

            let route = &self.routes[route_index];
            let count_fwd = vehicles_per_train_route / 2;
            let count_bwd = vehicles_per_train_route - count_fwd;

            // forward
            for i in 0..count_fwd {
                if route.stations.len() < 2 {
                    continue;
                }
                let last_station = 0;
                let next_station = 1;
                let f_lo = (i as f32) / (count_fwd as f32);
                let f_hi = ((i + 1) as f32) / (count_fwd as f32);
                let fraction = f_lo + (f_hi - f_lo) * rng();
                let speed = 0.005 + rng() * 0.01;
                let mut v = Vehicle {
                    vehicle_type: VehicleType::Train,
                    route_index,
                    direction: 1,
                    last_station,
                    next_station,
                    fraction,
                    speed,
                    x: 0.0,
                    y: 0.0,
                };
                v.update_position(route);
                self.vehicles.push(v);
            }

            // backward
            for i in 0..count_bwd {
                let n_stations = route.stations.len();
                if n_stations < 2 {
                    continue;
                }
                let last_station = n_stations - 1;
                let next_station = n_stations - 2;
                let f_lo = (i as f32) / (count_bwd as f32);
                let f_hi = ((i + 1) as f32) / (count_bwd as f32);
                let fraction = f_lo + (f_hi - f_lo) * rng();
                let speed = 0.005 + rng() * 0.01;
                let mut v = Vehicle {
                    vehicle_type: VehicleType::Train,
                    route_index,
                    direction: -1,
                    last_station,
                    next_station,
                    fraction,
                    speed,
                    x: 0.0,
                    y: 0.0,
                };
                v.update_position(route);
                self.vehicles.push(v);
            }
        }

        web_sys::console::log_1(
            &format!(
                "Created {} vehicles ({} buses, {} trains)",
                self.vehicles.len(),
                self.vehicles
                    .iter()
                    .filter(|v| v.vehicle_type == VehicleType::Bus)
                    .count(),
                self.vehicles
                    .iter()
                    .filter(|v| v.vehicle_type == VehicleType::Train)
                    .count()
            )
            .into(),
        );
    }

    pub fn update_all(&mut self) {
        for v in self.vehicles.iter_mut() {
            if v.route_index < self.routes.len() {
                let route = &self.routes[v.route_index];
                v.update_position(route);
            }
        }
    }
}

// Keep the global thread_local here too
thread_local! {
    pub static GLOBAL_STATE: RefCell<SharedState> = RefCell::new(SharedState::new());
}