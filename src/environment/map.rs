use noise::{NoiseFn, Perlin};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct MapConfig {
    pub width: usize,
    pub height: usize,
    pub seed: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellType {
    Empty,
    Obstacle,
    Energy,
    Mineral,
    ScientificSite,
}

#[derive(Debug)]
pub struct Map {
    pub config: MapConfig,
    pub cells: Vec<Vec<CellType>>,
    pub visibility: Vec<Vec<CellVisibility>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellVisibility {
    Hidden,
    Visible,
    Explored,
}

impl Map {
    pub fn new(config: MapConfig) -> Self {
        let mut map = Map {
            config: config.clone(),
            cells: vec![vec![CellType::Empty; config.width]; config.height],
            visibility: vec![vec![CellVisibility::Hidden; config.width]; config.height],
        };

        map.generate_terrain();
        map.clear_base_area();
        map.place_resources();

        let center_x = config.width / 2;
        let center_y = config.height / 2;
        map.update_visibility(center_x, center_y, 3);

        map
    }

    fn generate_terrain(&mut self) {
        let perlin = Perlin::new(self.config.seed);
        let scale = 0.15;

        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let val = perlin.get([x as f64 * scale, y as f64 * scale]);
                if val > 0.2 {
                    self.cells[y][x] = CellType::Obstacle;
                }
            }
        }

        let iterations = 4;
        for _ in 0..iterations {
            let mut new_cells = self.cells.clone();

            for y in 0..self.config.height {
                for x in 0..self.config.width {
                    let neighbors = self.count_obstacle_neighbors(x, y);

                    new_cells[y][x] = if self.cells[y][x] == CellType::Obstacle {
                        if neighbors >= 4 {
                            CellType::Obstacle
                        } else {
                            CellType::Empty
                        }
                    } else {
                        if neighbors >= 5 {
                            CellType::Obstacle
                        } else {
                            CellType::Empty
                        }
                    };
                }
            }

            self.cells = new_cells;
        }

        self.ensure_traversable();
    }

    fn count_obstacle_neighbors(&self, x: usize, y: usize) -> usize {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0
                    && nx < self.config.width as isize
                    && ny >= 0
                    && ny < self.config.height as isize
                {
                    if self.cells[ny as usize][nx as usize] == CellType::Obstacle {
                        count += 1;
                    }
                } else {
                    count += 1;
                }
            }
        }
        count
    }

    fn ensure_traversable(&mut self) {
        let mut rng = rand::thread_rng();
        let paths = 3;

        for _ in 0..paths {
            let start_x = rng.gen_range(0..self.config.width);
            let start_y = rng.gen_range(0..self.config.height);
            let end_x = rng.gen_range(0..self.config.width);
            let end_y = rng.gen_range(0..self.config.height);

            self.create_path(start_x, start_y, end_x, end_y);
        }
    }

    fn create_path(&mut self, start_x: usize, start_y: usize, end_x: usize, end_y: usize) {
        let mut x = start_x;
        let mut y = start_y;

        while x != end_x || y != end_y {
            self.cells[y][x] = CellType::Empty;

            if x < end_x {
                x += 1;
            } else if x > end_x {
                x -= 1;
            }

            if y < end_y {
                y += 1;
            } else if y > end_y {
                y -= 1;
            }

            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx >= 0
                        && nx < self.config.width as isize
                        && ny >= 0
                        && ny < self.config.height as isize
                    {
                        self.cells[ny as usize][nx as usize] = CellType::Empty;
                    }
                }
            }
        }
    }

    fn clear_base_area(&mut self) {
        let center_x = self.config.width / 2;
        let center_y = self.config.height / 2;

        for dy in -1..=1 {
            for dx in -1..=1 {
                let x = (center_x as isize + dx) as usize;
                let y = (center_y as isize + dy) as usize;

                if x < self.config.width && y < self.config.height {
                    self.cells[y][x] = CellType::Empty;
                }
            }
        }
    }

    pub fn place_resources(&mut self) {
        let mut rng = rand::thread_rng();

        let nb_energy = 20;
        let nb_minerals = 20;
        let nb_sites = 5;

        let is_valid_position = |x: usize, y: usize, map: &Map| -> bool {
            let center_x = map.config.width / 2;
            let center_y = map.config.height / 2;
            let dx = x.abs_diff(center_x);
            let dy = y.abs_diff(center_y);

            if dx <= 1 && dy <= 1 {
                return false;
            }

            map.cells[y][x] == CellType::Empty
        };

        for _ in 0..nb_energy {
            for _ in 0..10 {
                let x = rng.gen_range(0..self.config.width);
                let y = rng.gen_range(0..self.config.height);
                if is_valid_position(x, y, self) {
                    self.cells[y][x] = CellType::Energy;
                    break;
                }
            }
        }

        for _ in 0..nb_minerals {
            for _ in 0..10 {
                let x = rng.gen_range(0..self.config.width);
                let y = rng.gen_range(0..self.config.height);
                if is_valid_position(x, y, self) {
                    self.cells[y][x] = CellType::Mineral;
                    break;
                }
            }
        }

        for _ in 0..nb_sites {
            for _ in 0..10 {
                let x = rng.gen_range(0..self.config.width);
                let y = rng.gen_range(0..self.config.height);
                if is_valid_position(x, y, self) {
                    self.cells[y][x] = CellType::ScientificSite;
                    break;
                }
            }
        }
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        if x >= self.config.width || y >= self.config.height {
            return false;
        }
        self.cells[y][x] != CellType::Obstacle
    }

    pub fn update_visibility(&mut self, x: usize, y: usize, radius: i32) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let new_x = x as i32 + dx;
                let new_y = y as i32 + dy;

                if new_x >= 0
                    && new_x < self.config.width as i32
                    && new_y >= 0
                    && new_y < self.config.height as i32
                {
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    if distance <= radius as f32 {
                        let nx = new_x as usize;
                        let ny = new_y as usize;
                        self.visibility[ny][nx] = CellVisibility::Visible;
                    }
                }
            }
        }
    }

    pub fn fade_visibility(&mut self) {
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                if self.visibility[y][x] == CellVisibility::Visible {
                    self.visibility[y][x] = CellVisibility::Explored;
                }
            }
        }
    }
}
