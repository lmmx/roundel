// src/model/vehicle.rs

#[derive(Debug, PartialEq)]
pub enum VehicleType {
    Bus,
    Train,
}

pub struct Vehicle {
    pub vehicle_type: VehicleType,
    pub route_index: usize,
    pub direction: i8,
    pub last_station: usize,
    pub next_station: usize,
    pub fraction: f32,
    pub speed: f32,
    pub x: f32,
    pub y: f32,
}

impl Vehicle {
    pub fn update_position(&mut self, route: &crate::model::Route) {
        // (same code as before)
        self.fraction += self.speed;
        while self.fraction >= 1.0 {
            self.fraction -= 1.0;
            self.last_station = self.next_station;

            let next = (self.next_station as i32) + (self.direction as i32);
            if next < 0 || (next as usize) >= route.stations.len() {
                self.direction *= -1;
            }
            self.next_station = (self.last_station as i32 + self.direction as i32) as usize;
        }

        let (x1, y1) = route.stations[self.last_station];
        let (x2, y2) = route.stations[self.next_station];
        self.x = x1 + (x2 - x1) * self.fraction;
        self.y = y1 + (y2 - y1) * self.fraction;
    }
}
