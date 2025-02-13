use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use super::vehicle::Vehicle;

#[derive(Debug, Clone, Copy)]
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
    pub start: Position,
    pub end: Position,
    pub cost: i32,
    pub road_type: RoadType,
}

#[derive(Debug)]
pub struct Map {
    width: usize,
    height: usize,
    grid: Vec<Vec<Tile>>,
}

impl Map {
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

    pub fn find_path(&self, start: Position, goal: Position) -> Option<Vec<Position>> {
        let mut dist: HashMap<Position, i32> = HashMap::new();
        let mut prev: HashMap<Position, Position> = HashMap::new();
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

                let neighbor = Position {
                    x: nx as usize,
                    y: ny as usize,
                };

                // Only traverse if the neighbor is a Road tile.
                match self.grid[neighbor.y][neighbor.x].tile_type {
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

#[derive(PartialOrd, Ord, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}
