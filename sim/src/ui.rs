// src/ui.rs

use js_sys::Date;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, MouseEvent, Window, console};

use crate::model::{GLOBAL_STATE, VehicleType};

#[derive(Default)]
struct Camera {
    pub pan_x: f32,
    pub pan_y: f32,
}

/// Whether user is dragging, plus last mouse coords
#[derive(Default)]
struct DragState {
    pub is_dragging: bool,
    pub last_x: f32,
    pub last_y: f32,
}

// A thread-local "camera" storing our pan offset
thread_local! {
    static CAMERA: RefCell<Camera> = RefCell::new(Camera::default());
    static DRAG: RefCell<DragState> = RefCell::new(DragState::default());
}

/// Called once when the Wasm module loads:
/// 1) Initialize routes/vehicles
/// 2) Start the update loop (setInterval ~60 FPS)
/// 3) Attach mouse events so we can pan the canvas
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    // 1) Create routes & vehicles
    GLOBAL_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.init_vehicles(); // builds random routes + vehicles
    });

    // 2) Repeated update & draw
    start_animation_loop()?;

    // 3) Attach mouse listeners for panning
    attach_mouse_listeners()?;

    Ok(())
}

/// Creates a closure that runs at ~60 FPS (setInterval) and updates + draws.
fn start_animation_loop() -> Result<(), JsValue> {
    let closure = Closure::wrap(Box::new(move || {
        let t_start = Date::now();

        // Update vehicles
        GLOBAL_STATE.with(|cell| {
            cell.borrow_mut().update_all();
        });

        // Draw
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

/// Draw routes, offsetting by the camera's pan_x/pan_y
fn draw_routes(ctx: &CanvasRenderingContext2d) {
    // read the camera offset
    let (pan_x, pan_y) = CAMERA.with(|c| {
        let cam = c.borrow();
        (cam.pan_x, cam.pan_y)
    });

    GLOBAL_STATE.with(|cell| {
        let state = cell.borrow();
        for (i, route) in state.routes.iter().enumerate() {
            // color routes differently for trains vs buses
            if i < 10 {
                ctx.set_fill_style(&"rgba(255, 150, 150, 0.6)".into());
            } else {
                ctx.set_fill_style(&"rgba(150, 150, 255, 0.6)".into());
            }

            for &(sx, sy) in &route.stations {
                ctx.begin_path();
                // shift by pan_x / pan_y
                let draw_x = (sx - pan_x) as f64;
                let draw_y = (sy - pan_y) as f64;

                ctx.arc(draw_x, draw_y, 5.0, 0.0, 6.28).unwrap();
                ctx.fill();
            }
        }
    });
}

/// Draw vehicles, offsetting by camera
fn draw_vehicles(ctx: &CanvasRenderingContext2d) {
    let (pan_x, pan_y) = CAMERA.with(|c| {
        let cam = c.borrow();
        (cam.pan_x, cam.pan_y)
    });

    GLOBAL_STATE.with(|cell| {
        let state = cell.borrow();
        for v in &state.vehicles {
            ctx.begin_path();
            let draw_x = (v.x - pan_x) as f64;
            let draw_y = (v.y - pan_y) as f64;

            ctx.arc(draw_x, draw_y, 2.0, 0.0, 6.28).unwrap();

            match v.vehicle_type {
                VehicleType::Bus => ctx.set_fill_style(&"blue".into()),
                VehicleType::Train => ctx.set_fill_style(&"red".into()),
            }
            ctx.fill();
        }
    });
}

/// Attach mouse event listeners so the user can drag the canvas around.
fn attach_mouse_listeners() -> Result<(), JsValue> {
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
                        // we do "cam.pan_x -= dx" so dragging right -> negative offset
                        cam.pan_x -= dx;
                        cam.pan_y -= dy;
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
