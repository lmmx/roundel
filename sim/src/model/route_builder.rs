// src/model/route_builder.rs

use crate::model::Route;
use js_sys::Math;

pub fn build_random_train_routes() -> Vec<Route> {
    let rng = || (Math::random() * 1000.0) as f32;
    let mut train_routes = Vec::new();

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
        train_routes.push(Route { stations });
    }

    train_routes
}

pub fn build_random_bus_routes() -> Vec<Route> {
    let rng = || (Math::random() * 1000.0) as f32;
    let mut bus_routes = Vec::new();

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
        bus_routes.push(Route { stations });
    }

    bus_routes
}
