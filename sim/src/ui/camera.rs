// src/ui/camera.rs

use std::cell::RefCell;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, console, window};

/// Stores camera offset (pan_x, pan_y) and a zoom factor (`scale`).
#[derive(Default)]
pub struct Camera {
    pub pan_x: f32,
    pub pan_y: f32,
    pub scale: f32,
    pub selected_vehicle_index: Option<usize>,
    pub follow_mode: bool,
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
    pub static CAMERA: RefCell<Camera> = const { RefCell::new(Camera {
        pan_x: 0.0,
        pan_y: 0.0,
        scale: 1.0,
        selected_vehicle_index: None,
        follow_mode: false
    }) };

    pub static DRAG: RefCell<DragState> = RefCell::new(DragState::default());
}

/// Initialize the camera with values from the DOM inputs
pub fn initialize_camera_from_dom() {
    if let Some(window) = window() {
        if let Some(document) = window.document() {
            // Try to read pan_x
            if let Some(input_el) = document.get_element_by_id("cameraPanX") {
                if let Ok(input) = input_el.dyn_into::<HtmlInputElement>() {
                    if let Ok(value) = input.value().parse::<f32>() {
                        CAMERA.with(|c| {
                            c.borrow_mut().pan_x = value;
                        });
                    }
                }
            }

            // Try to read pan_y
            if let Some(input_el) = document.get_element_by_id("cameraPanY") {
                if let Ok(input) = input_el.dyn_into::<HtmlInputElement>() {
                    if let Ok(value) = input.value().parse::<f32>() {
                        CAMERA.with(|c| {
                            c.borrow_mut().pan_y = value;
                        });
                    }
                }
            }

            // Try to read scale
            if let Some(input_el) = document.get_element_by_id("cameraScale") {
                if let Ok(input) = input_el.dyn_into::<HtmlInputElement>() {
                    if let Ok(value) = input.value().parse::<f32>() {
                        // Ensure scale is at least 0.1 to avoid div by zero issues
                        let scale = if value < 0.1 { 0.1 } else { value };
                        CAMERA.with(|c| {
                            c.borrow_mut().scale = scale;
                        });
                    }
                }
            }

            // Log the camera state after initialization
            CAMERA.with(|c| {
                let cam = c.borrow();
                console::log_1(
                    &format!(
                        "Camera initialized: pan_x={}, pan_y={}, scale={}",
                        cam.pan_x, cam.pan_y, cam.scale
                    )
                    .into(),
                );
            });
        }
    }
}

/// Update the DOM inputs with current camera values
pub fn update_camera_dom_state() {
    if let Some(window) = window() {
        if let Some(document) = window.document() {
            CAMERA.with(|c| {
                let cam = c.borrow();

                // Update pan_x
                if let Some(input_el) = document.get_element_by_id("cameraPanX") {
                    if let Ok(input) = input_el.dyn_into::<HtmlInputElement>() {
                        input.set_value(&cam.pan_x.to_string());
                    }
                }

                // Update pan_y
                if let Some(input_el) = document.get_element_by_id("cameraPanY") {
                    if let Ok(input) = input_el.dyn_into::<HtmlInputElement>() {
                        input.set_value(&cam.pan_y.to_string());
                    }
                }

                // Update scale
                if let Some(input_el) = document.get_element_by_id("cameraScale") {
                    if let Ok(input) = input_el.dyn_into::<HtmlInputElement>() {
                        input.set_value(&cam.scale.to_string());
                    }
                }
            });
        }
    }
}
