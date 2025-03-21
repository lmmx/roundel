// src/model/route.rs

/// A route is a sequence of station coordinates (x, y).
pub struct Route {
    pub stations: Vec<(f32, f32)>,
}
