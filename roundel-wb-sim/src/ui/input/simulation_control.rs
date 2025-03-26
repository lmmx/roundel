// src/ui/input/simulation_control.rs

use crate::ui::camera::CAMERA;
use crate::ui::control::SIMULATION_CONTROL;
use crate::ui::core::{change_animation_interval, toggle_pause};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{Event, HtmlElement, HtmlInputElement, MouseEvent};

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

    // Follow vehicle button
    if let Some(follow_button) = document.get_element_by_id("followButton") {
        let follow_button = follow_button.dyn_into::<HtmlElement>()?;

        let closure_follow = Closure::wrap(Box::new(move |_: MouseEvent| {
            CAMERA.with(|c| {
                let mut cam = c.borrow_mut();
                // Only toggle if a vehicle is selected
                if cam.selected_vehicle_index.is_some() {
                    cam.follow_mode = !cam.follow_mode;

                    // Update button text
                    let button_text = if cam.follow_mode {
                        "Stop Following"
                    } else {
                        "Follow Selected Vehicle"
                    };

                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            if let Some(button) = document.get_element_by_id("followButton") {
                                button.set_text_content(Some(button_text));
                            }
                        }
                    }
                }
            });
        }) as Box<dyn FnMut(_)>);

        follow_button
            .add_event_listener_with_callback("click", closure_follow.as_ref().unchecked_ref())?;
        closure_follow.forget();
    }

    Ok(())
}
