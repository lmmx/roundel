// src/model/geo.rs

/// Utility for projecting geographical coordinates onto the canvas
pub struct GeoProjection {
    // Center of the map (lat, lng)
    center_lat: f32,
    center_lng: f32,
    // Scale factor (pixels per degree)
    pub scale: f32,
    // Canvas dimensions
    canvas_width: f32,
    canvas_height: f32,
}

impl GeoProjection {
    pub fn new(
        center_lat: f32,
        center_lng: f32,
        scale: f32,
        canvas_width: f32,
        canvas_height: f32,
    ) -> Self {
        Self {
            center_lat,
            center_lng,
            scale,
            canvas_width,
            canvas_height,
        }
    }

    /// London-centered projection with reasonable scale
    pub fn london_centered(canvas_width: f32, canvas_height: f32) -> Self {
        // London is roughly at 51.5, -0.12
        Self::new(51.5, -0.12, 5000.0, canvas_width, canvas_height)
    }

    /// Project a lat/lng coordinate to canvas x/y
    /// Returns (x, y) in pixels where (canvas_width/2, canvas_height/2) is the center
    pub fn project(&self, lat: f32, lng: f32) -> (f32, f32) {
        // Simple equirectangular projection
        // x = (λ - λ0) * cos(φ0) * scale
        // y = (φ0 - φ) * scale
        // where φ is latitude, λ is longitude, and φ0, λ0 is the center point

        // Convert to radians for cos
        let center_lat_rad = self.center_lat * std::f32::consts::PI / 180.0;

        // Calculate x, y offsets from center
        let x_offset = (lng - self.center_lng) * center_lat_rad.cos() * self.scale;
        let y_offset = (self.center_lat - lat) * self.scale;

        // Center on the canvas
        let x = self.canvas_width / 2.0 + x_offset;
        let y = self.canvas_height / 2.0 + y_offset;

        (x, y)
    }

    /// Project a series of coordinates
    pub fn project_many(&self, coords: &[(f32, f32)]) -> Vec<(f32, f32)> {
        coords
            .iter()
            .map(|&(lat, lng)| self.project(lat, lng))
            .collect()
    }

    /// Ensure a coordinate is within canvas bounds
    pub fn ensure_in_bounds(&self, coord: (f32, f32)) -> (f32, f32) {
        let (x, y) = coord;
        let margin = 50.0; // Keep points at least 50px from the edge

        let x_bounded = x.max(margin).min(self.canvas_width - margin);
        let y_bounded = y.max(margin).min(self.canvas_height - margin);

        (x_bounded, y_bounded)
    }
}

/// Parse a coordinate string in the format "lat,lng"
pub fn parse_coordinate(coord_str: &str) -> Result<(f32, f32), &'static str> {
    let parts: Vec<&str> = coord_str.split(',').collect();
    if parts.len() != 2 {
        return Err("Invalid coordinate format");
    }

    let lat = parts[0]
        .trim()
        .parse::<f32>()
        .map_err(|_| "Invalid latitude")?;
    let lng = parts[1]
        .trim()
        .parse::<f32>()
        .map_err(|_| "Invalid longitude")?;

    Ok((lat, lng))
}

/// Generates sample intermediate points between two coordinates
/// Simulates a route path with some natural curves instead of a straight line
pub fn generate_route_path(
    start: (f32, f32),
    end: (f32, f32),
    waypoints: usize,
    randomness: f32,
) -> Vec<(f32, f32)> {
    use js_sys::Math;

    let mut path = Vec::with_capacity(waypoints + 2);

    // Add start point
    path.push(start);

    // Generate intermediate waypoints
    if waypoints > 0 {
        // Linear interpolation with random offsets
        for i in 1..=waypoints {
            let t = i as f32 / (waypoints as f32 + 1.0);
            let base_x = start.0 + (end.0 - start.0) * t;
            let base_y = start.1 + (end.1 - start.1) * t;

            // Add random offset scaled by distance and randomness factor
            let dx = Math::random() as f32 * randomness * (end.0 - start.0).abs();
            let dy = Math::random() as f32 * randomness * (end.1 - start.1).abs();

            // Apply offset (randomly positive or negative)
            let x = base_x + if Math::random() > 0.5 { dx } else { -dx };
            let y = base_y + if Math::random() > 0.5 { dy } else { -dy };

            path.push((x, y));
        }
    }

    // Add end point
    path.push(end);

    path
}
