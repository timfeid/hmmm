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
    pub grid: Vec<Vec<Tile>>,
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

    pub fn display2(&self) {
        for y in (0..self.height) {
            let mut line = String::new();
            for x in 0..self.width {
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
            println!("{} {}", y, line);
        }
    }

    pub fn display(&self, vehicle: &Vehicle) {
        for y in (0..self.height) {
            let mut line = String::new();
            for x in 0..self.width {
                if vehicle.position.x as usize == x && vehicle.position.y as usize == y {
                    line.push('V');
                } else {
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
            println!("{} {}", y, line);
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

            for (dx, dy) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nx = pos.x + dx;
                let ny = pos.y + dy;

                if nx < 0 || ny < 0 || nx >= self.width as i32 || ny >= self.height as i32 {
                    continue;
                }

                let neighbor = Coordinates { x: nx, y: ny };

                match self.grid[neighbor.y as usize][neighbor.x as usize].tile_type {
                    TileType::Road(_) => {}
                    _ => continue,
                }

                let next_cost = cost + 1;
                if next_cost < *dist.get(&neighbor).unwrap_or(&i32::MAX) {
                    dist.insert(neighbor, next_cost);
                    prev.insert(neighbor, pos);
                    heap.push(Reverse((next_cost, neighbor)));
                }
            }
        }

        None
    }
}

impl Map {
    pub fn generate(&mut self) {
        let raw = vec![
            "==================================================",
            "  -              -               -             -  ",
            "  -              -               -             -  ",
            "  -              -               -             -  ",
            "  -              -               -             -  ",
            "  -              -               -             -  ",
            "  -++++++++++++++++++++++++++++++++++++++++++++-  ",
            "  -              +               +             -  ",
            "  -              +               +             -  ",
            "  -              +               +             -  ",
            "  -              +               +             -  ",
            "  -              +               +             -  ",
            "  -              +               +             -  ",
            "  -              +               +             -  ",
            "  -              +               +             -  ",
            "  -              +               +             -  ",
            "  -              +               +             -  ",
            "  -              +               +             -  ",
            "  -              +               +             -  ",
            "  -++++++++++++++++++++++++++++++++++++++++++++-  ",
            "  -              .               .             -  ",
            "  -              .               .             -  ",
            "  -  .............               .      ...    -  ",
            "  -       .      .               .      .      -  ",
            "  -       .      .................      .      -  ",
            "  -       .      .               .      .      -  ",
            "  -       .      .               .      .      -  ",
            "  -  ......      .               ........      -  ",
            "  -              .               .             -  ",
            "  -                                            -  ",
        ];

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

pub fn pixel_to_tile(tile: Coordinates) -> Coordinates {
    Coordinates {
        x: tile.x / 16,
        y: tile.y / 16,
    }
}

pub fn tile_to_pixel(tile: Coordinates) -> Coordinates {
    Coordinates {
        x: tile.x * 16,
        y: tile.y * 16,
    }
}

mod test {
    use crate::gangsta::vehicle::Vehicle;

    use super::Map;

    #[test]
    fn test() {
        let vehicle = Vehicle::new(
            "x".to_string(),
            super::Coordinates { x: 2, y: 2 },
            crate::gangsta::vehicle::VehicleBehavior::Aggressive,
            3,
        );
        let map =
            Map::from_json(include_str!("maps/suburb.json")).expect("unable to read json map?");
        map.display(&vehicle);
    }
}
