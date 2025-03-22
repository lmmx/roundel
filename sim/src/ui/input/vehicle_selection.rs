// src/ui/input/vehicle_selection.rs

use crate::model::GLOBAL_STATE;
use crate::ui::camera::CAMERA;
use crate::ui::camera::DRAG;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{HtmlCanvasElement, MouseEvent};

/// Attach event listener for vehicle tracking
pub fn attach_vehicle_selection_listener() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas_el = document
        .get_element_by_id("myCanvas")
        .ok_or("no #myCanvas element")?;

    let closure_click = Closure::wrap(Box::new(move |e: MouseEvent| {
        // Only handle clicks if not currently dragging
        let is_dragging = DRAG.with(|ds| ds.borrow().is_dragging);
        if is_dragging {
            return;
        }

        // Get a fresh reference to the canvas element inside the closure
        // This avoids consuming canvas_el which would make the closure FnOnce
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(canvas_el) = document.get_element_by_id("myCanvas") {
                    if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                        let rect = canvas.get_bounding_client_rect();
                        let click_x = e.client_x() as f32 - rect.left() as f32;
                        let click_y = e.client_y() as f32 - rect.top() as f32;

                        // Convert to world coordinates using camera
                        let (pan_x, pan_y, scale) = CAMERA.with(|c| {
                            let cam = c.borrow();
                            (cam.pan_x, cam.pan_y, cam.scale)
                        });

                        let world_x = click_x / scale + pan_x;
                        let world_y = click_y / scale + pan_y;

                        // Find closest vehicle
                        let mut closest_index = None;
                        let mut closest_dist = f32::MAX;

                        GLOBAL_STATE.with(|cell| {
                            let state = cell.borrow();
                            for (i, v) in state.vehicles.iter().enumerate() {
                                let dx = v.x - world_x;
                                let dy = v.y - world_y;
                                let dist_sq = dx * dx + dy * dy;

                                // Only select if within reasonable distance (adjusted for zoom)
                                let selection_threshold = 10.0 / scale;
                                if dist_sq < selection_threshold * selection_threshold
                                    && dist_sq < closest_dist
                                {
                                    closest_dist = dist_sq;
                                    closest_index = Some(i);
                                }
                            }
                        });

                        // Update selected vehicle
                        CAMERA.with(|c| {
                            let mut cam = c.borrow_mut();
                            cam.selected_vehicle_index = closest_index;

                            // Enable or disable the follow button based on selection
                            if let Some(window) = web_sys::window() {
                                if let Some(document) = window.document() {
                                    if let Some(follow_btn) =
                                        document.get_element_by_id("followButton")
                                    {
                                        if closest_index.is_some() {
                                            // Enable the button when vehicle is selected
                                            follow_btn.remove_attribute("disabled").ok();
                                        } else {
                                            // Disable the button when no vehicle is selected
                                            follow_btn.set_attribute("disabled", "true").ok();

                                            // Also turn off follow mode if it was on
                                            cam.follow_mode = false;
                                        }
                                    }
                                }
                            }
                        });
                    }
                }
            }
        }
    }) as Box<dyn FnMut(_)>);

    canvas_el.add_event_listener_with_callback("click", closure_click.as_ref().unchecked_ref())?;
    closure_click.forget();

    Ok(())
}
