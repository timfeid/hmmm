use std::{thread::sleep, time::Duration};

use rusty::gangsta::{
    map::{Coordinates, Map, Road},
    traffic_light::{TrafficLight, TrafficLightState},
    vehicle::{Vehicle, VehicleBehavior},
};

fn main() {
    let mut map = Map::new(50, 30);

    let start = Coordinates { x: 0, y: 0 };
    let destination = Coordinates { x: 42, y: 22 };

    let mut vehicle = Vehicle::new(1, start, destination, VehicleBehavior::Normal);
    if let Some(path) = map.find_path(start, destination) {
        println!("Path found: {:?}", path);
        vehicle.set_path(path);
        for tick in 0..150 {
            vehicle.move_step();
            map.display(&vehicle);
            sleep(Duration::from_millis(100));
        }
    } else {
        println!("No path found.");
    }
}
