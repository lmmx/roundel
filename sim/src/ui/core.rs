// src/ui/main_ui.rs

use super::draw::{draw_routes, draw_vehicles};
use super::input::{attach_mouse_listeners, attach_wheel_listener};
use crate::model::GLOBAL_STATE;
use js_sys::Date;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, console};

/// Called once when the Wasm module loads:
/// 1) Initialize routes/vehicles
/// 2) Start the update loop (setInterval ~60 FPS)
/// 3) Attach mouse events for panning and wheel event for zoom
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    // 1) Create routes & vehicles (random or real)
    GLOBAL_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.init_vehicles();
    });

    // 2) Repeated update & draw
    start_animation_loop()?;

    // 3) Attach mouse & wheel listeners
    attach_mouse_listeners()?;
    attach_wheel_listener()?;

    Ok(())
}

/// Creates a closure that runs at ~60 FPS (setInterval) and updates + draws.
fn start_animation_loop() -> Result<(), JsValue> {
    let closure = Closure::wrap(Box::new(move || {
        let t_start = Date::now();

        // 1) Update vehicles
        GLOBAL_STATE.with(|cell| {
            cell.borrow_mut().update_all();
        });

        // 2) Draw everything
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(canvas_el) = document.get_element_by_id("myCanvas") {
                    if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                        if let Ok(ctx) = canvas
                            .get_context("2d")
                            .unwrap()
                            .unwrap()
                            .dyn_into::<CanvasRenderingContext2d>()
                        {
                            let w = canvas.width() as f64;
                            let h = canvas.height() as f64;
                            ctx.clear_rect(0.0, 0.0, w, h);

                            draw_routes(&ctx);
                            draw_vehicles(&ctx);
                        }
                    }
                }
            }
        }

        let t_end = Date::now();
        let ms = t_end - t_start;
        console::log_1(&format!("Update & draw took {:.3} ms", ms).into());
    }) as Box<dyn FnMut()>);

    // setInterval(..., 16) => ~60 FPS
    let window = web_sys::window().unwrap();
    window.set_interval_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        16,
    )?;

    // Keep the closure alive
    closure.forget();

    Ok(())
}
