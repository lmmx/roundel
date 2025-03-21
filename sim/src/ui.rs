// src/wasm.rs

use wasm_bindgen::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, Window, Document, console
};
use js_sys::Date;

use crate::model::{GLOBAL_STATE, VehicleType};

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    // 1) Create routes & vehicles
    GLOBAL_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.init_vehicles();
    });

    // 2) Repeated update & draw
    let closure = Closure::wrap(Box::new(move || {
        let t_start = Date::now();

        // Update
        GLOBAL_STATE.with(|cell| {
            cell.borrow_mut().update_all();
        });

        // Draw
        let window: Window = web_sys::window().unwrap();
        let document: Document = window.document().unwrap();
        let canvas_el = document.get_element_by_id("myCanvas").unwrap();
        let canvas: HtmlCanvasElement = canvas_el.dyn_into().unwrap();
        let ctx = canvas
            .get_context("2d").unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>().unwrap();

        let w = canvas.width() as f64;
        let h = canvas.height() as f64;
        ctx.clear_rect(0.0, 0.0, w, h);

        // Draw routes as station circles
        GLOBAL_STATE.with(|cell| {
            let state = cell.borrow();
            for (i, route) in state.routes.iter().enumerate() {
                // color routes differently for trains vs buses
                if i < 10 {
                    ctx.set_fill_style_str("rgba(255, 150, 150, 0.6)");
                } else {
                    ctx.set_fill_style_str("rgba(150, 150, 255, 0.6)");
                }

                for &(sx, sy) in &route.stations {
                    ctx.begin_path();
                    ctx.arc(sx as f64, sy as f64, 5.0, 0.0, 6.28).unwrap();
                    ctx.fill();
                }
            }

            // Draw vehicles
            for v in &state.vehicles {
                ctx.begin_path();
                ctx.arc(v.x as f64, v.y as f64, 2.0, 0.0, 6.28).unwrap();
                match v.vehicle_type {
                    VehicleType::Bus => {
                        ctx.set_fill_style_str("blue");
                    }
                    VehicleType::Train => {
                        ctx.set_fill_style_str("red");
                    }
                }
                ctx.fill();
            }
        });

        let t_end = Date::now();
        let ms = t_end - t_start;
        console::log_1(&format!("Update & draw took {:.3} ms", ms).into());
    }) as Box<dyn FnMut()>);

    // 3) setInterval(closure, 16 ms => ~60 FPS)
    let window = web_sys::window().unwrap();
    window.set_interval_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        16,
    )?;

    // Prevent closure from being dropped
    closure.forget();

    Ok(())
}
