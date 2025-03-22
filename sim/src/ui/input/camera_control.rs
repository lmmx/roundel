// src/ui/input/camera_control.rs

use crate::ui::camera::{CAMERA, DRAG, update_camera_dom_state};
use std::cell::RefCell;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{Document, MouseEvent, WheelEvent};

/// Attach mouse events so user can pan the canvas.
pub fn attach_mouse_listeners() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document: Document = window.document().unwrap();
    let canvas_el = document
        .get_element_by_id("myCanvas")
        .ok_or("no #myCanvas element")?;

    // MOUSE DOWN
    {
        let closure_mousedown = Closure::wrap(Box::new(move |e: MouseEvent| {
            DRAG.with(|ds| {
                let mut drag = ds.borrow_mut();
                drag.is_dragging = true;
                drag.last_x = e.client_x() as f32;
                drag.last_y = e.client_y() as f32;
            });
        }) as Box<dyn FnMut(_)>);

        canvas_el.add_event_listener_with_callback(
            "mousedown",
            closure_mousedown.as_ref().unchecked_ref(),
        )?;
        closure_mousedown.forget();
    }

    // MOUSE UP
    {
        let closure_mouseup = Closure::wrap(Box::new(move |_e: MouseEvent| {
            DRAG.with(|ds| {
                ds.borrow_mut().is_dragging = false;
            });

            // When mouse up happens, update the DOM state
            update_camera_dom_state();
        }) as Box<dyn FnMut(_)>);

        // We can add mouseup to the canvas or the entire document
        document.add_event_listener_with_callback(
            "mouseup",
            closure_mouseup.as_ref().unchecked_ref(),
        )?;
        closure_mouseup.forget();
    }

    // MOUSE MOVE
    {
        let closure_mousemove = Closure::wrap(Box::new(move |e: MouseEvent| {
            let new_x = e.client_x() as f32;
            let new_y = e.client_y() as f32;

            DRAG.with(|ds| {
                let mut drag = ds.borrow_mut();
                if drag.is_dragging {
                    // compute delta
                    let dx = new_x - drag.last_x;
                    let dy = new_y - drag.last_y;

                    // shift camera
                    CAMERA.with(|c| {
                        let mut cam = c.borrow_mut();
                        // dragging right => negative offset
                        cam.pan_x -= dx / cam.scale; // incorporate scale
                        cam.pan_y -= dy / cam.scale;
                    });

                    // update last
                    drag.last_x = new_x;
                    drag.last_y = new_y;
                }
            });
        }) as Box<dyn FnMut(_)>);

        document.add_event_listener_with_callback(
            "mousemove",
            closure_mousemove.as_ref().unchecked_ref(),
        )?;
        closure_mousemove.forget();
    }

    Ok(())
}

/// Attach a wheel event to allow zoom in/out with the mouse wheel.
pub fn attach_wheel_listener() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas_el = document
        .get_element_by_id("myCanvas")
        .ok_or("no #myCanvas element")?;

    let closure_wheel = Closure::wrap(Box::new(move |e: WheelEvent| {
        e.prevent_default(); // prevent page scroll
        let delta = e.delta_y(); // positive => zoom out, negative => zoom in

        CAMERA.with(|c| {
            let mut cam = c.borrow_mut();

            if delta < 0.0 {
                // zoom in
                cam.scale *= 1.1;
            } else {
                // zoom out
                cam.scale *= 0.9;
            }

            // clamp scale
            if cam.scale < 0.1 {
                cam.scale = 0.1;
            } else if cam.scale > 50.0 {
                cam.scale = 50.0;
            }
        });

        // Update DOM state after zoom
        update_camera_dom_state();
    }) as Box<dyn FnMut(_)>);

    canvas_el.add_event_listener_with_callback("wheel", closure_wheel.as_ref().unchecked_ref())?;
    closure_wheel.forget();

    Ok(())
}

/// Attach touch events to allow pinch-to-zoom on touchscreens.
pub fn attach_touch_listeners() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas_el = document
        .get_element_by_id("myCanvas")
        .ok_or("no #myCanvas element")?;

    // Use thread_local for previous touch distance
    thread_local! {
        static PREV_TOUCH_DISTANCE: RefCell<f64> = RefCell::new(0.0);
    }

    // TOUCH START
    {
        let closure_touchstart = Closure::wrap(Box::new(move |e: web_sys::TouchEvent| {
            e.prevent_default(); // Prevent default actions

            // If we have exactly 2 touches, store the initial distance for pinch detection
            if e.touches().length() == 2 {
                let touch1 = e.touches().get(0).unwrap();
                let touch2 = e.touches().get(1).unwrap();

                let dx = touch1.client_x() - touch2.client_x();
                let dy = touch1.client_y() - touch2.client_y();
                let distance = ((dx * dx + dy * dy) as f64).sqrt();
                PREV_TOUCH_DISTANCE.with(|cell| {
                    *cell.borrow_mut() = distance;
                });
            }

            // If we have a single touch, set up for panning (similar to mouse down)
            if e.touches().length() == 1 {
                let touch = e.touches().get(0).unwrap();
                DRAG.with(|ds| {
                    let mut drag = ds.borrow_mut();
                    drag.is_dragging = true;
                    drag.last_x = touch.client_x() as f32;
                    drag.last_y = touch.client_y() as f32;
                });
            }
        }) as Box<dyn FnMut(_)>);

        canvas_el.add_event_listener_with_callback(
            "touchstart",
            closure_touchstart.as_ref().unchecked_ref(),
        )?;
        closure_touchstart.forget();
    }

    // TOUCH MOVE
    {
        let closure_touchmove = Closure::wrap(Box::new(move |e: web_sys::TouchEvent| {
            e.prevent_default(); // Prevent scrolling

            // Handle pinch-to-zoom with two fingers
            if e.touches().length() == 2 {
                let touch1 = e.touches().get(0).unwrap();
                let touch2 = e.touches().get(1).unwrap();

                let dx = touch1.client_x() - touch2.client_x();
                let dy = touch1.client_y() - touch2.client_y();
                let current_distance = ((dx * dx + dy * dy) as f64).sqrt();

                // Only proceed if we have a valid previous distance
                PREV_TOUCH_DISTANCE.with(|cell| {
                    let prev_distance = *cell.borrow();
                    if prev_distance > 0.0 {
                        // Calculate zoom factor
                        let zoom_factor = current_distance / prev_distance;

                        CAMERA.with(|c| {
                            let mut cam = c.borrow_mut();

                            // Apply zoom (similar to wheel handler)
                            if zoom_factor > 1.0 {
                                // Zoom in
                                cam.scale *= zoom_factor as f32;
                            } else if zoom_factor < 1.0 {
                                // Zoom out
                                cam.scale *= zoom_factor as f32;
                            }

                            // Clamp scale to reasonable bounds
                            if cam.scale < 0.1 {
                                cam.scale = 0.1;
                            } else if cam.scale > 50.0 {
                                cam.scale = 50.0;
                            }
                        });
                    }

                    // Update for next move event
                    *cell.borrow_mut() = current_distance;
                });
            }

            // Handle panning with one finger (similar to mousemove)
            if e.touches().length() == 1 {
                let touch = e.touches().get(0).unwrap();
                let new_x = touch.client_x() as f32;
                let new_y = touch.client_y() as f32;

                DRAG.with(|ds| {
                    let mut drag = ds.borrow_mut();
                    if drag.is_dragging {
                        // Compute delta
                        let dx = new_x - drag.last_x;
                        let dy = new_y - drag.last_y;

                        // Shift camera
                        CAMERA.with(|c| {
                            let mut cam = c.borrow_mut();
                            cam.pan_x -= dx / cam.scale;
                            cam.pan_y -= dy / cam.scale;
                        });

                        // Update last position
                        drag.last_x = new_x;
                        drag.last_y = new_y;
                    }
                });
            }
        }) as Box<dyn FnMut(_)>);

        canvas_el.add_event_listener_with_callback(
            "touchmove",
            closure_touchmove.as_ref().unchecked_ref(),
        )?;
        closure_touchmove.forget();
    }

    // TOUCH END
    {
        let closure_touchend = Closure::wrap(Box::new(move |e: web_sys::TouchEvent| {
            e.prevent_default();

            // Reset drag state when touch ends
            if e.touches().length() == 0 {
                DRAG.with(|ds| {
                    ds.borrow_mut().is_dragging = false;
                });

                // Reset touch distance
                PREV_TOUCH_DISTANCE.with(|cell| {
                    *cell.borrow_mut() = 0.0;
                });

                // Update DOM state
                update_camera_dom_state();
            }
        }) as Box<dyn FnMut(_)>);

        canvas_el.add_event_listener_with_callback(
            "touchend",
            closure_touchend.as_ref().unchecked_ref(),
        )?;
        closure_touchend.forget();
    }

    // TOUCH CANCEL (handle unexpected cancellations)
    {
        let closure_touchcancel = Closure::wrap(Box::new(move |e: web_sys::TouchEvent| {
            e.prevent_default();

            // Reset drag state
            DRAG.with(|ds| {
                ds.borrow_mut().is_dragging = false;
            });

            // Reset touch distance
            PREV_TOUCH_DISTANCE.with(|cell| {
                *cell.borrow_mut() = 0.0;
            });

            // Update DOM state
            update_camera_dom_state();
        }) as Box<dyn FnMut(_)>);

        canvas_el.add_event_listener_with_callback(
            "touchcancel",
            closure_touchcancel.as_ref().unchecked_ref(),
        )?;
        closure_touchcancel.forget();
    }

    Ok(())
}
