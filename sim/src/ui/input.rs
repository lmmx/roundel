// src/ui/input.rs

use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{Document, MouseEvent, WheelEvent};
use super::camera::{CAMERA, DRAG};

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
    }) as Box<dyn FnMut(_)>);

    canvas_el.add_event_listener_with_callback("wheel", closure_wheel.as_ref().unchecked_ref())?;
    closure_wheel.forget();

    Ok(())
}
