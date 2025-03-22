// src/ui/input.rs

use super::camera::{CAMERA, DRAG};
use super::control::SIMULATION_CONTROL;
use super::core::{change_animation_interval, toggle_pause};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{Document, Event, HtmlElement, HtmlInputElement, MouseEvent, WheelEvent};

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

/// Attach event listeners for the simulation controls
pub fn attach_control_listeners() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Pause button
    if let Some(pause_button) = document.get_element_by_id("pauseButton") {
        let pause_button = pause_button.dyn_into::<HtmlElement>()?;

        let closure_pause = Closure::wrap(Box::new(move |_: MouseEvent| {
            let is_paused = toggle_pause();

            // Update button text based on pause state
            let button_text = if is_paused { "Resume" } else { "Pause" };

            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(button) = document.get_element_by_id("pauseButton") {
                        button.set_text_content(Some(button_text));
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        pause_button
            .add_event_listener_with_callback("click", closure_pause.as_ref().unchecked_ref())?;
        closure_pause.forget();
    }

    // FPS radio buttons
    if let Some(fps_30) = document.get_element_by_id("fps30") {
        let fps_30 = fps_30.dyn_into::<HtmlInputElement>()?;

        let closure_fps30 = Closure::wrap(Box::new(move |_: Event| {
            let _ = change_animation_interval(33); // ~30 FPS
        }) as Box<dyn FnMut(_)>);

        fps_30
            .add_event_listener_with_callback("change", closure_fps30.as_ref().unchecked_ref())?;
        closure_fps30.forget();
    }

    if let Some(fps_60) = document.get_element_by_id("fps60") {
        let fps_60 = fps_60.dyn_into::<HtmlInputElement>()?;

        let closure_fps60 = Closure::wrap(Box::new(move |_: Event| {
            let _ = change_animation_interval(16); // ~60 FPS
        }) as Box<dyn FnMut(_)>);

        fps_60
            .add_event_listener_with_callback("change", closure_fps60.as_ref().unchecked_ref())?;
        closure_fps60.forget();
    }

    // Auto FPS checkbox
    if let Some(auto_fps) = document.get_element_by_id("autoFps") {
        let auto_fps = auto_fps.dyn_into::<HtmlInputElement>()?;

        let closure_auto = Closure::wrap(Box::new(move |e: Event| {
            if let Some(target) = e.target() {
                if let Ok(checkbox) = target.dyn_into::<HtmlInputElement>() {
                    let checked = checkbox.checked();

                    SIMULATION_CONTROL.with(|cell| {
                        let mut control = cell.borrow_mut();
                        control.auto_adjust = checked;

                        // If auto-adjust is enabled, reset the frame counter and times
                        if checked {
                            control.frame_count = 0;
                            control.total_frame_time = 0.0;
                        }
                    });

                    // Enable/disable the FPS radio buttons based on auto setting
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            if let Some(fps30) = document.get_element_by_id("fps30") {
                                if let Ok(radio) = fps30.dyn_into::<HtmlInputElement>() {
                                    radio.set_disabled(checked);
                                }
                            }
                            if let Some(fps60) = document.get_element_by_id("fps60") {
                                if let Ok(radio) = fps60.dyn_into::<HtmlInputElement>() {
                                    radio.set_disabled(checked);
                                }
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        auto_fps
            .add_event_listener_with_callback("change", closure_auto.as_ref().unchecked_ref())?;
        closure_auto.forget();
    }

    Ok(())
}
