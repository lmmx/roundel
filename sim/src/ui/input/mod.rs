// src/ui/input/mod.rs

mod camera_control;
mod simulation_control;
mod vehicle_selection;

pub use camera_control::{attach_mouse_listeners, attach_touch_listeners, attach_wheel_listener};
pub use simulation_control::attach_control_listeners;
pub use vehicle_selection::attach_vehicle_selection_listener;
