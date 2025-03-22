// src/ui/draw.rs

use std::f64::consts::TAU;
use super::camera::CAMERA;
use super::control::get_vehicle_counts;
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
            // Skip routes with fewer than 2 stations
            if route.stations.len() < 2 {
                continue;
            }

            // Draw the line connecting all stations
            ctx.begin_path();

            // Get the first point
            let (first_x, first_y) = route.stations[0];
            let draw_first_x = (first_x - pan_x) * scale;
            let draw_first_y = (first_y - pan_y) * scale;

            ctx.move_to(draw_first_x as f64, draw_first_y as f64);

            // Connect all stations with lines
            for &(sx, sy) in route.stations.iter().skip(1) {
                let draw_x = (sx - pan_x) * scale;
                let draw_y = (sy - pan_y) * scale;
                ctx.line_to(draw_x as f64, draw_y as f64);
            }

            // Set stroke style based on route type
            if i < 10 {
                // Train routes
                ctx.set_stroke_style_str("rgba(255, 0, 0, 0.4)");
            } else {
                // Bus routes
                ctx.set_stroke_style_str("rgba(0, 0, 255, 0.4)");
            }

            ctx.set_line_width(2.0 * scale as f64);
            ctx.stroke();

            // Now draw stations as circles
            for (idx, &(sx, sy)) in route.stations.iter().enumerate() {
                ctx.begin_path();

                // Shift by pan_x / pan_y, then apply scale
                let draw_x = (sx - pan_x) * scale;
                let draw_y = (sy - pan_y) * scale;

                // Determine circle size:
                // - For train routes (i < 10): All stations are the same size (5.0)
                // - For bus routes (i >= 10): Terminals are larger (5.0), intermediate stops are smaller (3.0)
                let is_terminal = i >= 10 && (idx == 0 || idx == route.stations.len() - 1);
                let circle_size = if is_terminal || i < 10 { 5.0 } else { 2.0 };

                ctx.arc(
                    draw_x as f64,
                    draw_y as f64,
                    circle_size * scale as f64,
                    0.0,
                    2.0 * std::f64::consts::PI,
                )
                .unwrap();

                // Set fill color based on route type
                if i < 10 {
                    // Train stations
                    ctx.set_fill_style_str("rgba(255, 100, 100, 0.7)");
                } else {
                    // Bus stops - terminals are darker
                    if is_terminal {
                        ctx.set_fill_style_str("rgba(50, 50, 255, 0.9)");
                    } else {
                        ctx.set_fill_style_str("rgba(100, 100, 255, 0.6)");
                    }
                }

                ctx.fill();
            }
        }
    });
}

/// Draw vehicles, offset by camera pan and scale
pub fn draw_vehicles(ctx: &CanvasRenderingContext2d) {
    let (pan_x, pan_y, scale, selected_index) = CAMERA.with(|c| {
        let cam = c.borrow();
        (cam.pan_x, cam.pan_y, cam.scale, cam.selected_vehicle_index)
    });

    GLOBAL_STATE.with(|cell| {
        let state = cell.borrow();
        for (i, v) in state.vehicles.iter().enumerate() {
            ctx.begin_path();

            let draw_x = (v.x - pan_x) * scale;
            let draw_y = (v.y - pan_y) * scale;

            // Is this the selected vehicle?
            let is_selected = selected_index == Some(i);
            
            // Draw the vehicle with appropriate styling
            let radius = if is_selected {
                3.0 * scale as f64  // Bigger for selected vehicle
            } else {
                2.0 * scale as f64
            };
            
            ctx.arc(draw_x as f64, draw_y as f64, radius, 0.0, TAU).unwrap();

            // Color based on vehicle type and selection
            match v.vehicle_type {
                VehicleType::Bus => {
                    if is_selected {
                        ctx.set_fill_style_str("lime")  // Highlight selected bus
                    } else {
                        ctx.set_fill_style_str("blue")
                    }
                },
                VehicleType::Train => {
                    if is_selected {
                        ctx.set_fill_style_str("orange")  // Highlight selected train
                    } else {
                        ctx.set_fill_style_str("red")
                    }
                },
            }
            ctx.fill();
            
            // Draw a highlight ring around selected vehicle
            if is_selected {
                ctx.begin_path();
                ctx.arc(draw_x as f64, draw_y as f64, 6.0 * scale as f64, 0.0, TAU).unwrap();
                ctx.set_stroke_style_str("yellow");
                ctx.set_line_width(2.0);
                ctx.stroke();
            }
        }
    });
}

/// Draw statistics about the simulation (vehicle counts, etc.)
pub fn draw_stats(ctx: &CanvasRenderingContext2d) {
    // Get vehicle counts
    let counts = get_vehicle_counts();

    // Save context state
    ctx.save();

    // Set text properties
    ctx.set_font("16px Arial");
    ctx.set_fill_style_str("black");

    // Draw background for stats area
    ctx.set_fill_style_str("rgba(255, 255, 255, 0.7)");
    ctx.fill_rect(10.0, 10.0, 200.0, 80.0);

    // Draw text
    ctx.set_fill_style_str("black");
    ctx.fill_text("Vehicles in transit:", 20.0, 30.0).unwrap();
    ctx.fill_text(&format!("Buses: {}", counts.buses), 30.0, 50.0)
        .unwrap();
    ctx.fill_text(&format!("Trains: {}", counts.trains), 30.0, 70.0)
        .unwrap();
    ctx.fill_text(&format!("Total: {}", counts.total), 130.0, 70.0)
        .unwrap();

    // Restore context state
    ctx.restore();
}
