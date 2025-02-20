use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    error::Error,
    fs::read_to_string,
};

use serde::{Deserialize, Serialize};
use specta::Type;

use super::vehicle::Vehicle;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct MapConfig {
    pub compressionlevel: i32,
    pub height: usize,
    pub infinite: bool,
    pub layers: Vec<Layer>,
    pub nextlayerid: usize,
    pub nextobjectid: usize,
    pub orientation: String,
    pub renderorder: String,
    pub tiledversion: String,
    pub tileheight: usize,
    pub tilesets: Vec<Tileset>,
    pub tilewidth: usize,
    pub r#type: String,
    pub version: String,
    pub width: usize,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Layer {
    pub chunks: Vec<Chunk>,
    pub height: usize,
    pub id: usize,
    pub name: String,
    pub opacity: i32,
    pub startx: i32,
    pub starty: i32,
    pub r#type: String,
    pub visible: bool,
    pub width: usize,
    pub x: i32,
    pub y: i32,
    pub offsetx: Option<f64>,
    pub offsety: Option<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Chunk {
    pub data: Vec<i32>,
    pub height: usize,
    pub width: usize,
    pub x: i32,
    pub y: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Tileset {
    pub columns: usize,
    pub firstgid: usize,
    pub image: String,
    pub imageheight: usize,
    pub imagewidth: usize,
    pub margin: usize,
    pub name: String,
    pub spacing: usize,
    pub tilecount: usize,
    pub tileheight: usize,
    pub tilewidth: usize,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TileType {
    Empty,
    Road(RoadType),
    Building,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub tile_type: TileType,
}

impl Tile {
    pub fn new(tile_type: TileType) -> Self {
        Tile { tile_type }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum RoadType {
    Interstate,
    Arterial,
    Collector,
    Local,
}

#[derive(Debug)]
pub struct Road {
    pub start: Coordinates,
    pub end: Coordinates,
    pub cost: i32,
    pub road_type: RoadType,
}

#[derive(Debug, Clone)]
pub struct Map {
    width: usize,
    height: usize,
    grid: Vec<Vec<Tile>>,
}

impl Map {
    pub fn from_json(str: &str) -> Result<Map, Box<dyn std::error::Error>> {
        let tiled_map: MapConfig = serde_json::from_str(str)?;
        let road_layer = tiled_map
            .layers
            .into_iter()
            .find(|layer| layer.name.to_lowercase() == "road")
            .ok_or("No 'road' layer found in the map")?;
        let mut min_x: isize = isize::MAX;
        let mut min_y: isize = isize::MAX;
        let mut max_x: isize = isize::MIN;
        let mut max_y: isize = isize::MIN;
        for chunk in &road_layer.chunks {
            let cx = chunk.x as isize;
            let cy = chunk.y as isize;
            if cx < min_x {
                min_x = cx;
            }
            if cy < min_y {
                min_y = cy;
            }
            let chunk_max_x = cx + chunk.width as isize;
            let chunk_max_y = cy + chunk.height as isize;
            if chunk_max_x > max_x {
                max_x = chunk_max_x;
            }
            if chunk_max_y > max_y {
                max_y = chunk_max_y;
            }
        }
        let grid_width = (max_x - min_x) as usize;
        let grid_height = (max_y - min_y) as usize;
        let mut grid: Vec<Vec<Tile>> =
            vec![vec![Tile::new(TileType::Empty); grid_width]; grid_height];
        for chunk in &road_layer.chunks {
            for (index, &tile_id) in chunk.data.iter().enumerate() {
                let x = chunk.x as isize + (index % chunk.width) as isize;
                let y = chunk.y as isize + (index / chunk.width) as isize;
                let grid_x = (x - min_x) as usize;
                let grid_y = (y - min_y) as usize;
                let tile = if tile_id == 0 {
                    Tile::new(TileType::Empty)
                } else {
                    Tile::new(TileType::Road(RoadType::Local))
                };
                grid[grid_y][grid_x] = tile;
            }
        }
        Ok(Map {
            width: grid_width,
            height: grid_height,
            grid,
        })
    }

    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![Tile::new(TileType::Empty); width]; height];
        let mut map = Map {
            width,
            height,
            grid,
        };

        map.generate();

        map
    }

    pub fn set_road(&mut self, x: usize, y: usize, road_type: RoadType) {
        if x < self.width && y < self.height {
            if let TileType::Empty = self.grid[y][x].tile_type {
                self.grid[y][x] = Tile::new(TileType::Road(road_type));
            }
        }
    }

    pub fn set_building(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            if let TileType::Empty = self.grid[y][x].tile_type {
                self.grid[y][x] = Tile::new(TileType::Building);
            }
        }
    }

    pub fn display(&self, vehicle: &Vehicle) {
        // Iterate over rows from top (highest index) to bottom (0)
        for y in (0..self.height) {
            let mut line = String::new();
            for x in 0..self.width {
                // If this cell is where the vehicle is located, show the vehicle.
                if vehicle.position.x as usize == x && vehicle.position.y as usize == y {
                    line.push('V');
                } else {
                    // Otherwise, show the tile based on its type.
                    let ch = match self.grid[y][x].tile_type {
                        TileType::Empty => ' ',
                        TileType::Road(RoadType::Interstate) => '□',
                        TileType::Road(RoadType::Arterial) => '□',
                        TileType::Road(RoadType::Collector) => '□',
                        TileType::Road(RoadType::Local) => '□',
                        TileType::Building => ' ',
                    };
                    line.push(ch);
                }
            }
            println!("{}", line);
        }
    }

    pub fn find_path(&self, start: Coordinates, goal: Coordinates) -> Option<Vec<Coordinates>> {
        let mut dist: HashMap<Coordinates, i32> = HashMap::new();
        let mut prev: HashMap<Coordinates, Coordinates> = HashMap::new();
        let mut heap = BinaryHeap::new();

        dist.insert(start, 0);
        heap.push(Reverse((0, start)));

        while let Some(Reverse((cost, pos))) = heap.pop() {
            if pos == goal {
                // Reconstruct path from goal back to start.
                let mut path = Vec::new();
                let mut current = pos;
                while let Some(&p) = prev.get(&current) {
                    path.push(current);
                    current = p;
                }
                path.push(start);
                path.reverse();
                return Some(path);
            }

            // Check 4 neighbors (up, down, left, right)
            for (dx, dy) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nx = pos.x as i32 + dx;
                let ny = pos.y as i32 + dy;

                if nx < 0 || ny < 0 || nx >= self.width as i32 || ny >= self.height as i32 {
                    continue;
                }

                let neighbor = Coordinates {
                    x: nx as i32,
                    y: ny as i32,
                };

                // Only traverse if the neighbor is a Road tile.
                if neighbor.y > 0 || neighbor.x > 0 {
                    continue;
                }
                match self.grid[neighbor.y as usize][neighbor.x as usize].tile_type {
                    TileType::Road(_) => { /* valid */ }
                    _ => continue,
                }

                let next_cost = cost + 1; // uniform cost for each step
                if next_cost < *dist.get(&neighbor).unwrap_or(&i32::MAX) {
                    dist.insert(neighbor, next_cost);
                    prev.insert(neighbor, pos);
                    heap.push(Reverse((next_cost, neighbor)));
                }
            }
        }

        // No path found.
        None
    }
}

impl Map {
    pub fn generate(&mut self) {
        // 30 rows × 50 columns. (Make sure each string is exactly 50 characters.)
        let raw = vec![
            "==================================================", // row  0
            "  -              -               -             -  ", // row  1
            "  -              -               -             -  ", // row  2: Arterials (horizontal)
            "  -              -               -             -  ", // row  3
            "  -              -               -             -  ", // row  4
            "  -              -               -             -  ", // row  5
            "  -++++++++++++++++++++++++++++++++++++++++++++-  ", // row  6: Collectors
            "  -              +               +             -  ", // row  7: Locals
            "  -              +               +             -  ", // row  8
            "  -              +               +             -  ", // row  9
            "  -              +               +             -  ", // row 10
            "  -              +               +             -  ", // row 11: More arterials (horizontal)
            "  -              +               +             -  ", // row 12
            "  -              +               +             -  ", // row 13
            "  -              +               +             -  ", // row 14: Interstate (horizontal)
            "  -              +               +             -  ", // row 15: Interstate (thick)
            "  -              +               +             -  ", // row 16
            "  -              +               +             -  ", // row 17
            "  -              +               +             -  ", // row 18
            "  -++++++++++++++++++++++++++++++++++++++++++++-  ", // row 19
            "  -              .               .             -  ", // row 20: Arterials (again)
            "  -              .               .             -  ", // row 21
            "  -  .............               .      ...    -  ", // row 22: Collectors
            "  -       .      .               .      .      -  ", // row 23: Locals
            "  -       .      .................      .      -  ", // row 24: Locals
            "  -       .      .               .      .      -  ", // row 25
            "  -       .      .               .      .      -  ", // row 26: Vertical Interstate (using "  ")
            "  -  ......      .               ........      -  ", // row 27
            "  -              .               .             -  ", // row 28
            "  -                                            -  ", // row 29
        ];

        // Convert each character into a Tile using our mapping:
        // ' ' -> Empty
        // '=' or '|' -> Road(Interstate)
        // '-' -> Road(Arterial)
        // '+' -> Road(Collector)
        // '.' -> Road(Local)
        // '#' -> Building
        let response = raw
            .into_iter()
            .map(|line| {
                line.chars()
                    .map(|c| match c {
                        ' ' => Tile::new(TileType::Empty),
                        '=' | '|' => Tile::new(TileType::Road(RoadType::Interstate)),
                        '-' => Tile::new(TileType::Road(RoadType::Arterial)),
                        '+' => Tile::new(TileType::Road(RoadType::Collector)),
                        '.' => Tile::new(TileType::Road(RoadType::Local)),
                        '#' => Tile::new(TileType::Building),
                        _ => Tile::new(TileType::Empty),
                    })
                    .collect()
            })
            .collect();
        self.grid = response;
    }
}

#[derive(
    Type, Serialize, Deserialize, PartialOrd, Ord, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}

mod test {
    use crate::gangsta::vehicle::Vehicle;

    use super::Map;

    // #[tokio::test]
    #[test]
    fn test() {
        let vehicle = Vehicle::new(
            0,
            super::Coordinates { x: 2, y: 2 },
            super::Coordinates { x: 55, y: 55 },
            crate::gangsta::vehicle::VehicleBehavior::Aggressive,
        );
        let map =
            Map::from_json(include_str!("maps/suburb.json")).expect("unable to read json map?");
        map.display(&vehicle);
    }
}
