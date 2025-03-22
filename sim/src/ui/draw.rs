// src/ui/draw.rs

use super::camera::CAMERA;
use crate::model::{GLOBAL_STATE, VehicleType};
use web_sys::CanvasRenderingContext2d;

/// Draw routes, offsetting by the camera's pan_x/pan_y and scaling by camera.scale
pub fn draw_routes(ctx: &CanvasRenderingContext2d) {
    let (pan_x, pan_y, scale) = CAMERA.with(|c| {
        let cam = c.borrow();
        (cam.pan_x, cam.pan_y, cam.scale)
    });

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

                // shift by pan_x / pan_y, then apply scale
                let draw_x = (sx - pan_x) * scale;
                let draw_y = (sy - pan_y) * scale;

                ctx.arc(draw_x as f64, draw_y as f64, 5.0 * scale as f64, 0.0, 6.28)
                    .unwrap();
                ctx.fill();
            }
        }
    });
}

/// Draw vehicles, offset by camera pan and scale
pub fn draw_vehicles(ctx: &CanvasRenderingContext2d) {
    let (pan_x, pan_y, scale) = CAMERA.with(|c| {
        let cam = c.borrow();
        (cam.pan_x, cam.pan_y, cam.scale)
    });

    GLOBAL_STATE.with(|cell| {
        let state = cell.borrow();
        for v in &state.vehicles {
            ctx.begin_path();

            let draw_x = (v.x - pan_x) * scale;
            let draw_y = (v.y - pan_y) * scale;

            // vehicles are small, so also multiply the circle radius by scale
            ctx.arc(draw_x as f64, draw_y as f64, 2.0 * scale as f64, 0.0, 6.28)
                .unwrap();

            match v.vehicle_type {
                VehicleType::Bus => ctx.set_fill_style_str("blue"),
                VehicleType::Train => ctx.set_fill_style_str("red"),
            }
            ctx.fill();
        }
    });
}
