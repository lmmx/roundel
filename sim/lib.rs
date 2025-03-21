use console_error_panic_hook as set_panic_hook;
use js_sys::{Math, Date};
use std::cell::RefCell;
use wasm_bindgen::{
    prelude::*,
    closure::Closure,
    JsCast,
};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, console, Window, Document};

/// A route is a sequence of station coordinates (x, y). Each route might represent
/// a train line or a bus line. Vehicles travel station-to-station along this route.
struct Route {
    /// The list of (x, y) station coordinates in the route. A vehicle traveling
    /// between station `i` and station `i+1` linearly interpolates its position.
    stations: Vec<(f32, f32)>,
}

/// The possible types of vehicles in our simulation (Bus vs. Train).
#[derive(Debug, PartialEq)]
enum VehicleType {
    /// A bus traveling on one of our "bus routes"
    Bus,
    /// A train traveling on one of our "train routes"
    Train,
}

/// A vehicle traveling along a route, moving from station to station. 
/// 
/// # Fields
/// - `vehicle_type`: whether it's a bus or train
/// - `route_index`: index into `SharedState::routes`, indicating which route this vehicle is on
/// - `direction`: +1 or -1, indicating forward or backward travel through the route's station list
/// - `last_station` and `next_station`: indices of the current and next station
/// - `fraction`: a value in [0..1) indicating how far we've traveled between `last_station` and `next_station`
/// - `speed`: how much `fraction` we advance each update
/// - `(x, y)`: the current position on the canvas, computed via interpolation
#[derive(Debug)]
struct Vehicle {
    /// Whether this is a Bus or Train
    vehicle_type: VehicleType,
    /// Which route (by index in `SharedState.routes`) we're traveling on
    route_index: usize,
    /// +1 or -1 to indicate forward/backward direction
    direction: i8,
    /// The station index we last passed
    last_station: usize,
    /// The next station index we're traveling toward
    next_station: usize,
    /// Progress between stations (0..1)
    fraction: f32,
    /// Amount of fraction to move each update
    speed: f32,
    /// Current x-coordinate on the canvas
    x: f32,
    /// Current y-coordinate on the canvas
    y: f32,
}

impl Vehicle {
    /// Advance this vehicle's position along its route by `speed`. If `fraction` exceeds 1.0, 
    /// that means we've arrived at `next_station`: update `last_station` to `next_station`, 
    /// compute a new `next_station` (possibly reversing direction if we're at the route end), 
    /// and interpolate (x, y) between those stations.
    ///
    /// # Arguments
    ///
    /// * `route` - The route data (`Route`) that this vehicle is traveling on
    fn update_position(&mut self, route: &Route) {
        // Advance fraction
        self.fraction += self.speed;
        while self.fraction >= 1.0 {
            self.fraction -= 1.0;

            // We reached the next station
            self.last_station = self.next_station;
            let next = (self.next_station as i32) + (self.direction as i32);
            // If we're about to go out of bounds, flip direction
            if next < 0 || (next as usize) >= route.stations.len() {
                self.direction *= -1;
            }
            self.next_station = (self.last_station as i32 + self.direction as i32) as usize;
        }

        // Interpolate the station coords to find (x, y)
        let (x1, y1) = route.stations[self.last_station];
        let (x2, y2) = route.stations[self.next_station];
        self.x = x1 + (x2 - x1) * self.fraction;
        self.y = y1 + (y2 - y1) * self.fraction;
    }
}

/// The global simulation state, which holds:
/// - A list of routes (`routes`), each containing station coordinates
/// - A list of vehicles (`vehicles`), each referencing a route
///
/// This struct can build random routes, create bus/train vehicles, update them,
/// and provide data for rendering.
struct SharedState {
    /// A list of all routes in the simulation. The first 10 are train routes,
    /// and the next 100 are bus routes (though the code doesn't strictly enforce that).
    routes: Vec<Route>,
    /// All vehicles currently in the simulation (both buses and trains).
    vehicles: Vec<Vehicle>,
}

impl SharedState {
    /// Construct a new, empty `SharedState`.
    fn new() -> Self {
        Self {
            routes: Vec::new(),
            vehicles: Vec::new(),
        }
    }

    /// Build 110 routes total:
    /// - First 10 are "train" routes (vertical-ish).
    /// - Next 100 are "bus" routes (horizontal-ish).
    ///
    /// We store them in `self.routes`.
    fn build_routes(&mut self) {
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
                stations.push((
                    x_base + x_off,
                    y_start + y_step * (i as f32) + y_off,
                ));
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
                stations.push((
                    x_start + x_step * (i as f32) + x_off,
                    y_base + y_off,
                ));
            }
            self.routes.push(Route { stations });
        }
    }

    /// Initialize all routes and create 2,000 vehicles:
    /// - 1,000 buses spread across 100 bus routes (10 per route)
    /// - 1,000 trains spread across 10 train routes (100 per route)
    ///
    /// Each vehicle is set up with forward or backward direction, a random
    /// fraction within a segment so vehicles won't overlap, and a random speed.
    fn init_vehicles(&mut self) {
        self.build_routes();
        let rng = || Math::random() as f32;

        // 100 bus routes => each route gets 10 vehicles (5 forward, 5 backward)
        let bus_start_index = 10;
        let num_bus_routes = 100;
        let vehicles_per_bus_route = 10;

        for r_i in 0..num_bus_routes {
            let route_index = bus_start_index + r_i;
            let route = &self.routes[route_index];
            let count_fwd = vehicles_per_bus_route / 2;
            let count_bwd = vehicles_per_bus_route - count_fwd;

            // Forward vehicles
            for i in 0..count_fwd {
                if route.stations.len() < 2 { continue; }
                let last_station = 0;
                let next_station = 1;
                // Subdivide [0..1] to avoid overlapping
                let f_lo = (i as f32) / (count_fwd as f32);
                let f_hi = ((i + 1) as f32) / (count_fwd as f32);
                let fraction = f_lo + (f_hi - f_lo) * rng();

                let speed = 0.005 + rng() * 0.01;
                let mut vehicle = Vehicle {
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
                vehicle.update_position(route);
                self.vehicles.push(vehicle);
            }

            // Backward vehicles
            for i in 0..count_bwd {
                let n_stations = route.stations.len();
                if n_stations < 2 { continue; }
                let last_station = n_stations - 1;
                let next_station = n_stations - 2;
                let f_lo = (i as f32) / (count_bwd as f32);
                let f_hi = ((i + 1) as f32) / (count_bwd as f32);
                let fraction = f_lo + (f_hi - f_lo) * rng();

                let speed = 0.005 + rng() * 0.01;
                let mut vehicle = Vehicle {
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
                vehicle.update_position(route);
                self.vehicles.push(vehicle);
            }
        }

        // 10 train routes => each route gets 100 vehicles (50 forward, 50 backward)
        let train_routes = 10;
        let vehicles_per_train_route = 100;
        for r_i in 0..train_routes {
            let route_index = r_i;
            let route = &self.routes[route_index];
            let count_fwd = vehicles_per_train_route / 2;
            let count_bwd = vehicles_per_train_route - count_fwd;

            // Forward
            for i in 0..count_fwd {
                if route.stations.len() < 2 { continue; }
                let last_station = 0;
                let next_station = 1;
                let f_lo = (i as f32) / (count_fwd as f32);
                let f_hi = ((i + 1) as f32) / (count_fwd as f32);
                let fraction = f_lo + (f_hi - f_lo) * rng();

                let speed = 0.005 + rng() * 0.01;
                let mut vehicle = Vehicle {
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
                vehicle.update_position(route);
                self.vehicles.push(vehicle);
            }

            // Backward
            for i in 0..count_bwd {
                let n_stations = route.stations.len();
                if n_stations < 2 { continue; }
                let last_station = n_stations - 1;
                let next_station = n_stations - 2;
                let f_lo = (i as f32) / (count_bwd as f32);
                let f_hi = ((i + 1) as f32) / (count_bwd as f32);
                let fraction = f_lo + (f_hi - f_lo) * rng();

                let speed = 0.005 + rng() * 0.01;
                let mut vehicle = Vehicle {
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
                vehicle.update_position(route);
                self.vehicles.push(vehicle);
            }
        }
    }

    /// Update every vehicle by calling its `update_position`. This moves them
    /// between stations, flipping direction if they hit a route end.
    fn update_all(&mut self) {
        for v in self.vehicles.iter_mut() {
            let route = &self.routes[v.route_index];
            v.update_position(route);
        }
    }
}

// A thread-local global storing our entire simulation state. In single-threaded 
// Wasm, this is effectively just a single global, but we don't have to mark it `Sync`.
thread_local! {
    static GLOBAL_STATE: RefCell<SharedState> = RefCell::new(SharedState::new());
}

/// The main entry point, called when the Wasm is loaded. We initialize routes/vehicles,
/// then set up a repeating timer (via `setInterval`) to update positions and draw 
/// everything onto an HTML canvas.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    set_panic_hook::set_once();

    // 1) Build routes and create vehicles
    GLOBAL_STATE.with(|cell| {
        cell.borrow_mut().init_vehicles();
    });

    // 2) Create a closure that runs repeatedly to:
    //   (a) Update positions
    //   (b) Draw everything
    let closure = Closure::wrap(Box::new(move || {
        let t_start = Date::now();

        // a) Update positions
        GLOBAL_STATE.with(|cell| {
            cell.borrow_mut().update_all();
        });

        // b) Draw to canvas
        let window: Window = web_sys::window().expect("no global `window`");
        let document: Document = window.document().expect("should have a document");
        let canvas_el = document
            .get_element_by_id("myCanvas")
            .expect("document should have a #myCanvas");
        let canvas: HtmlCanvasElement = canvas_el
            .dyn_into()
            .expect("Failed to cast to HtmlCanvasElement");
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let w = canvas.width() as f64;
        let h = canvas.height() as f64;
        ctx.clear_rect(0.0, 0.0, w, h);

        // Draw stations first
        GLOBAL_STATE.with(|cell| {
            let state = cell.borrow();
            for (route_index, route) in state.routes.iter().enumerate() {
                // Distinguish bus vs train routes by index
                if route_index < 10 {
                    // Train route: light red
                    ctx.set_fill_style_str("rgba(255, 150, 150, 0.6)");
                } else {
                    // Bus route: light blue
                    ctx.set_fill_style_str("rgba(150, 150, 255, 0.6)");
                }

                for &(sx, sy) in &route.stations {
                    ctx.begin_path();
                    ctx.arc(sx as f64, sy as f64, 5.0, 0.0, 6.28).unwrap();
                    ctx.fill();
                }
            }
        });

        // Draw vehicles on top
        GLOBAL_STATE.with(|cell| {
            let state = cell.borrow();
            for v in &state.vehicles {
                ctx.begin_path();
                ctx.arc(v.x as f64, v.y as f64, 2.0, 0.0, 6.28).unwrap();
                match v.vehicle_type {
                    VehicleType::Bus => {
                        ctx.set_fill_style_str("blue");
                    }
                    VehicleType::Train => {
                        ctx.set_fill_style_str("red");
                    }
                }
                ctx.fill();
            }
        });

        let t_end = Date::now();
        let ms = t_end - t_start;
        console::log_1(&format!("Update & draw took {:.3} ms", ms).into());
    }) as Box<dyn FnMut()>);

    // 3) Request updates ~60 times per second (16 ms intervals)
    web_sys::window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            16,
        )?;

    // Keep the closure alive
    closure.forget();

    Ok(())
}
