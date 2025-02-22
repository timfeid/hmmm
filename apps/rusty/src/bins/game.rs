use std::{thread::sleep, time::Duration};

use rusty::gangsta::{
    map::{Coordinates, Map, Road},
    traffic_light::{TrafficLight, TrafficLightState},
    vehicle::{Vehicle, VehicleBehavior},
};

fn main() {
    let mut map = Map::from_json(include_str!("../gangsta/maps/suburb.json")).expect("no");

    let start = Coordinates { x: 20, y: 20 };
    let destination = Coordinates { x: 21, y: 20 };

    let mut vehicle = Vehicle::new("".to_string(), start, VehicleBehavior::Normal, 5);
    map.display(&vehicle);
    println!("{:?}", map.grid[start.x as usize][start.y as usize]);
    println!("{:?}", map.grid[(start.x + 1) as usize][start.y as usize]);
    println!("{:?}", map.grid[20][20]);
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
