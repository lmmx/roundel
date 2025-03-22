// src/ui/camera.rs

use std::cell::RefCell;

/// Stores camera offset (pan_x, pan_y) and a zoom factor (`scale`).
#[derive(Default)]
pub struct Camera {
    pub pan_x: f32,
    pub pan_y: f32,
    pub scale: f32,
}

/// Whether user is dragging, plus last mouse coords
#[derive(Default)]
pub struct DragState {
    pub is_dragging: bool,
    pub last_x: f32,
    pub last_y: f32,
}

// A thread-local "camera" storing our pan offset and zoom scale.
thread_local! {
    pub static CAMERA: RefCell<Camera> = RefCell::new(Camera {
        pan_x: 0.0,
        pan_y: 0.0,
        scale: 1.0
    });

    pub static DRAG: RefCell<DragState> = RefCell::new(DragState::default());
}
