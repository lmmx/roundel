// src/model/mod.rs

pub mod geo;
pub mod route;
pub mod route_builder;
pub mod state;
pub mod vehicle;

pub use route::Route;
pub use state::{GLOBAL_STATE, SharedState};
pub use vehicle::{Vehicle, VehicleType};