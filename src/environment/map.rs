use noise::{NoiseFn, Perlin};
use rand::Rng;

/// Configuration de la carte (dimensions, seed, etc.)
#[derive(Debug, Clone)]  // Added Clone
pub struct MapConfig {
    pub width: usize,
    pub height: usize,
    pub seed: u32,
    // etc.
}

/// Représente les différents types de cases ou ressources de la carte.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellType {
    Empty,
    Obstacle,
    Energy,
    Mineral,
    ScientificSite,
}

/// Structure de la map
#[derive(Debug)]
pub struct Map {
    pub config: MapConfig,
    pub cells: Vec<Vec<CellType>>, // 2D grid
}

impl Map {
    /// Construit une nouvelle carte en fonction d'une configuration
    pub fn new(config: MapConfig) -> Self {
        let mut map = Map {
            config: config.clone(),
            cells: vec![vec![CellType::Empty; config.width]; config.height],
        };

        map.generate_terrain();
        map.clear_base_area();
        map.place_resources();
        map
    }

    /// Generate terrain using Perlin noise and cellular automata
    fn generate_terrain(&mut self) {
        // First pass: Perlin noise for initial terrain
        let perlin = Perlin::new(self.config.seed);
        let scale = 0.15; // Increased scale for more varied terrain

        // Initialize with Perlin noise
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let val = perlin.get([x as f64 * scale, y as f64 * scale]);
                if val > 0.2 { // Lower threshold for more obstacles
                    self.cells[y][x] = CellType::Obstacle;
                }
            }
        }

        // Second pass: Cellular automata for more natural-looking terrain
        let iterations = 4;
        for _ in 0..iterations {
            let mut new_cells = self.cells.clone();
            
            for y in 0..self.config.height {
                for x in 0..self.config.width {
                    let neighbors = self.count_obstacle_neighbors(x, y);
                    
                    new_cells[y][x] = if self.cells[y][x] == CellType::Obstacle {
                        // Stay obstacle if enough neighbors are obstacles
                        if neighbors >= 4 {
                            CellType::Obstacle
                        } else {
                            CellType::Empty
                        }
                    } else {
                        // Become obstacle if enough neighbors are obstacles
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

        // Final pass: Ensure map is traversable
        self.ensure_traversable();
    }

    /// Count number of obstacle neighbors (including diagonals)
    fn count_obstacle_neighbors(&self, x: usize, y: usize) -> usize {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0 && nx < self.config.width as isize &&
                   ny >= 0 && ny < self.config.height as isize {
                    if self.cells[ny as usize][nx as usize] == CellType::Obstacle {
                        count += 1;
                    }
                } else {
                    // Count out-of-bounds as obstacles
                    count += 1;
                }
            }
        }
        count
    }

    /// Ensure the map is traversable by creating paths
    fn ensure_traversable(&mut self) {
        let mut rng = rand::thread_rng();
        let paths = 3; // Number of random paths to create

        for _ in 0..paths {
            let start_x = rng.gen_range(0..self.config.width);
            let start_y = rng.gen_range(0..self.config.height);
            let end_x = rng.gen_range(0..self.config.width);
            let end_y = rng.gen_range(0..self.config.height);

            self.create_path(start_x, start_y, end_x, end_y);
        }
    }

    /// Create a path between two points using a simple line algorithm
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

            // Clear a small area around the path
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx >= 0 && nx < self.config.width as isize &&
                       ny >= 0 && ny < self.config.height as isize {
                        self.cells[ny as usize][nx as usize] = CellType::Empty;
                    }
                }
            }
        }
    }

    /// Clear the area around the base to ensure it's accessible
    fn clear_base_area(&mut self) {
        let center_x = self.config.width / 2;
        let center_y = self.config.height / 2;
        
        // Clear a 3x3 area around the base
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

    /// Placement simpliste de ressources
    pub fn place_resources(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Nombre d'items à générer (ex. 5 de chaque)
        let nb_energy = 20;
        let nb_minerals = 20;
        let nb_sites = 5;

        // Helper function to check if position is valid for resource placement
        let is_valid_position = |x: usize, y: usize, map: &Map| -> bool {
            let center_x = map.config.width / 2;
            let center_y = map.config.height / 2;
            let dx = x.abs_diff(center_x);
            let dy = y.abs_diff(center_y);
            
            // Don't place resources near the base
            if dx <= 1 && dy <= 1 {
                return false;
            }
            
            map.cells[y][x] == CellType::Empty
        };
    
        // Énergie
        for _ in 0..nb_energy {
            for _ in 0..10 { // Try up to 10 times to find a valid position
                let x = rng.gen_range(0..self.config.width);
                let y = rng.gen_range(0..self.config.height);
                if is_valid_position(x, y, self) {
                    self.cells[y][x] = CellType::Energy;
                    break;
                }
            }
        }
    
        // Minerais
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
    
        // Sites scientifiques
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
    

    /// Vérifie si une position est valide et non-obstacle
    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        if x >= self.config.width || y >= self.config.height {
            return false;
        }
        self.cells[y][x] != CellType::Obstacle
    }
}
