// src/model.rs

use js_sys::Math;
use std::cell::RefCell;

/// A route is a sequence of station coordinates (x, y). 
pub struct Route {
    pub stations: Vec<(f32, f32)>,
}

#[derive(Debug, PartialEq)]
pub enum VehicleType {
    Bus,
    Train,
}

pub struct Vehicle {
    pub vehicle_type: VehicleType,
    pub route_index: usize,
    pub direction: i8,
    pub last_station: usize,
    pub next_station: usize,
    pub fraction: f32,
    pub speed: f32,
    pub x: f32,
    pub y: f32,
}

impl Vehicle {
    /// Interpolate the station coords to update (x, y).
    pub fn update_position(&mut self, route: &Route) {
        // Move fraction
        self.fraction += self.speed;
        while self.fraction >= 1.0 {
            self.fraction -= 1.0;
            self.last_station = self.next_station;

            // Move to next station, flipping if out of bounds
            let next = (self.next_station as i32) + (self.direction as i32);
            if next < 0 || (next as usize) >= route.stations.len() {
                self.direction *= -1;
            }
            self.next_station = (self.last_station as i32 + self.direction as i32) as usize;
        }

        let (x1, y1) = route.stations[self.last_station];
        let (x2, y2) = route.stations[self.next_station];
        self.x = x1 + (x2 - x1) * self.fraction;
        self.y = y1 + (y2 - y1) * self.fraction;
    }
}

/// The global simulation state, storing all routes and vehicles.
pub struct SharedState {
    pub routes: Vec<Route>,
    pub vehicles: Vec<Vehicle>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            vehicles: Vec::new(),
        }
    }

    /// Build 110 routes: 10 train + 100 bus routes
    pub fn build_routes(&mut self) {
        let rng = || (Math::random() * 1000.0) as f32;

        // 10 "train" routes
        for _ in 0..10 {
            let mut stations = Vec::new();
            let x_base = rng();
            let y_start = rng();
            let y_step = 100.0 + 50.0 * (Math::random() as f32);
            for i in 0..6 {
                let x_off = (Math::random() as f32) * 30.0 - 15.0;
                let y_off = (Math::random() as f32) * 30.0 - 15.0;
                stations.push((x_base + x_off, y_start + y_step * (i as f32) + y_off));
            }
            self.routes.push(Route { stations });
        }

        // 100 "bus" routes
        for _ in 0..100 {
            let mut stations = Vec::new();
            let y_base = rng();
            let x_start = rng();
            let x_step = 80.0 + 50.0 * (Math::random() as f32);
            for i in 0..8 {
                let x_off = (Math::random() as f32) * 20.0 - 10.0;
                let y_off = (Math::random() as f32) * 20.0 - 10.0;
                stations.push((x_start + x_step * (i as f32) + x_off, y_base + y_off));
            }
            self.routes.push(Route { stations });
        }
    }

    /// Create the vehicles: 1,000 buses + 1,000 trains
    pub fn init_vehicles(&mut self) {
        self.build_routes();
        let rng = || Math::random() as f32;

        // Buses
        let bus_start_index = 10;
        let num_bus_routes = 100;
        let vehicles_per_bus_route = 10;

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

        // Trains
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

    /// Update every vehicle.
    pub fn update_all(&mut self) {
        for v in self.vehicles.iter_mut() {
            let route = &self.routes[v.route_index];
            v.update_position(route);
        }
    }
}

// A single-thread “global” store for our simulation.
thread_local! {
    pub static GLOBAL_STATE: RefCell<SharedState> = RefCell::new(SharedState::new());
}
