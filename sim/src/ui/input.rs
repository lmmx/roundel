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

    // Get the current FPS value from the control
    let initial_fps = SIMULATION_CONTROL.with(|cell| {
        let control = cell.borrow();
        if control.update_interval_ms > 0 {
            1000 / control.update_interval_ms
        } else {
            30 // Default
        }
    });

    // Set the initial FPS display value
    if let Some(fps_value) = document.get_element_by_id("fpsValue") {
        fps_value.set_text_content(Some(&format!("{} FPS", initial_fps)));
    }

    // Also set the slider to match the current FPS value
    if let Some(fps_slider) = document.get_element_by_id("fpsSlider") {
        if let Ok(slider) = fps_slider.dyn_into::<HtmlInputElement>() {
            slider.set_value(&initial_fps.to_string());
        }
    }

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

    // FPS slider
    if let Some(fps_slider) = document.get_element_by_id("fpsSlider") {
        let fps_slider = fps_slider.dyn_into::<HtmlInputElement>()?;

        let closure_fps_slider = Closure::wrap(Box::new(move |e: Event| {
            if let Some(target) = e.target() {
                if let Ok(slider) = target.dyn_into::<HtmlInputElement>() {
                    if let Ok(fps) = slider.value().parse::<u32>() {
                        // Convert FPS to interval in milliseconds
                        let interval_ms = if fps > 0 { 1000 / fps } else { 33 }; // Default to ~30 FPS if invalid

                        // Update the FPS value display
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                if let Some(fps_value) = document.get_element_by_id("fpsValue") {
                                    fps_value.set_text_content(Some(&format!("{} FPS", fps)));
                                }
                            }
                        }

                        let _ = change_animation_interval(interval_ms);
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        // Listen for both input (live updates while dragging) and change (final value)
        fps_slider.add_event_listener_with_callback(
            "input",
            closure_fps_slider.as_ref().unchecked_ref(),
        )?;

        closure_fps_slider.forget();
    }

    Ok(())
}
