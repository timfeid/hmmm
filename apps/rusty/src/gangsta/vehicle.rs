use std::collections::VecDeque;

use super::map::Coordinates;

#[derive(Debug, Clone, Copy)]
pub enum VehicleBehavior {
    Cautious,   // Slower speed, stops early at red lights
    Normal,     // Regular driving behavior
    Aggressive, // Ignores some rules (runs yellow lights)
}

#[derive(Debug)]
pub struct Vehicle {
    pub id: usize,
    pub position: Coordinates,
    pub destination: Coordinates,
    speed: f32, // Units per tick
    behavior: VehicleBehavior,
    path: VecDeque<Coordinates>, // Route to follow
}

impl Vehicle {
    pub fn new(
        id: usize,
        start: Coordinates,
        destination: Coordinates,
        behavior: VehicleBehavior,
    ) -> Self {
        Self {
            id,
            position: start,
            destination,
            speed: match behavior {
                VehicleBehavior::Cautious => 2.0,
                VehicleBehavior::Normal => 4.0,
                VehicleBehavior::Aggressive => 6.0,
            },
            behavior,
            path: VecDeque::new(), // Empty for now, will be assigned a route
        }
    }

    pub fn move_step(&mut self) {
        if let Some(next_position) = self.path.pop_front() {
            self.position = next_position;
        }
    }

    pub fn set_path(&mut self, path: Vec<Coordinates>) -> &Vehicle {
        self.path = VecDeque::from(path);

        self
    }
}
