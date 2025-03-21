// src/model/state.rs

use crate::model::route_builder::{build_random_bus_routes, build_random_train_routes};
use crate::model::{Route, Vehicle, VehicleType};
use std::cell::RefCell;

pub struct SharedState {
    pub routes: Vec<Route>,
    pub vehicles: Vec<Vehicle>,
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
        }
    }

    /// Called by init_vehicles(); sets up random routes.
    fn build_random_routes(&mut self) {
        // fetch random train routes
        let mut trains = build_random_train_routes();
        // fetch random bus routes
        let mut buses = build_random_bus_routes();

        // combine them
        self.routes.clear();
        self.routes.append(&mut trains); // first 10 => trains
        self.routes.append(&mut buses); // next 100 => buses
    }

    /// Create the vehicles: 1,000 buses + 1,000 trains
    pub fn init_vehicles(&mut self) {
        // Instead of directly doing random, call our helper:
        self.build_random_routes();

        let rng = || js_sys::Math::random() as f32;

        // Buses => route indices 10..(10+100)
        let bus_start_index = 10;
        let num_bus_routes = 100;
        let vehicles_per_bus_route = 10;

        // (same code as before, referencing self.routes)
        for r_i in 0..num_bus_routes {
            let route_index = bus_start_index + r_i;
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

        // Trains => route indices 0..10
        let train_routes = 10;
        let vehicles_per_train_route = 100;

        for r_i in 0..train_routes {
            let route_index = r_i;
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
    }

    pub fn update_all(&mut self) {
        for v in self.vehicles.iter_mut() {
            let route = &self.routes[v.route_index];
            v.update_position(route);
        }
    }
}

// Keep the global thread_local here too
thread_local! {
    pub static GLOBAL_STATE: RefCell<SharedState> = RefCell::new(SharedState::new());
}
