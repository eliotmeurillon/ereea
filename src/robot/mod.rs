use rand::Rng;
use crate::environment::Map;
use crate::environment::map::CellType;
use crate::station::Station;
use crate::pathfinding;

/// Différents modules spécialisés qu'un robot peut embarquer
#[derive(Debug, PartialEq)]
pub enum RobotModule {
    Exploration,        
    Drill,              
    EnergyCollector,    
    // Removed unused variants ChemicalAnalyzer and HighResCamera
}

/// Un robot peut avoir plusieurs modules
#[derive(Debug)]
pub struct Robot {
    // Unique identifier for debugging and future features
    pub id: usize,
    pub x: usize,
    pub y: usize,
    pub modules: Vec<RobotModule>,
    pub has_data_to_share: bool,

    pub carried_energy: u32,
    pub carried_minerals: u32,
    pub carried_scientific_data: u32,
}

impl Robot {
    /// Exemple de constructeur
    pub fn new(id: usize, x: usize, y: usize, modules: Vec<RobotModule>) -> Self {
        Self {
            id,
            x,
            y,
            modules,
            has_data_to_share: false,
            carried_energy: 0,
            carried_minerals: 0,
            carried_scientific_data: 0,
        }
    }

    /// Check if the robot should return to base
    fn should_return_to_base(&self) -> bool {
        self.carried_energy > 0 || 
        self.carried_minerals > 0 || 
        self.carried_scientific_data > 0
    }

    /// Move towards a target position using A* pathfinding
    fn move_towards(&mut self, target_x: usize, target_y: usize, map: &Map) {
        let start = (self.x, self.y);
        let goal = (target_x, target_y);

        if let Some(path) = pathfinding::find_path(map, start, goal) {
            if path.len() > 1 {  // If we have a path and aren't already at the goal
                let next_pos = path[1];  // Get the next position in the path
                self.x = next_pos.0;
                self.y = next_pos.1;
            }
        } else {
            // Fallback to simple movement if no path is found
            let dx = if self.x < target_x { 1 } else if self.x > target_x { -1 } else { 0 };
            let dy = if self.y < target_y { 1 } else if self.y > target_y { -1 } else { 0 };

            let new_x = (self.x as isize + dx).clamp(0, map.config.width as isize - 1) as usize;
            let new_y = (self.y as isize + dy).clamp(0, map.config.height as isize - 1) as usize;

            if map.is_walkable(new_x, new_y) {
                self.x = new_x;
                self.y = new_y;
            }
        }
    }

    /// Check if the robot is at or adjacent to the base
    fn is_near_base(&self, center_x: usize, center_y: usize) -> bool {
        let dx = self.x.abs_diff(center_x);
        let dy = self.y.abs_diff(center_y);
        dx <= 1 && dy <= 1  // Within 1 tile of the base (including diagonally)
    }

    /// Updated movement logic - stop when near base
    pub fn random_move(&mut self, map: &Map) {
        let center_x = map.config.width / 2;
        let center_y = map.config.height / 2;

        if self.should_return_to_base() {
            // Only move towards base if not already adjacent
            if !self.is_near_base(center_x, center_y) {
                self.move_towards(center_x, center_y, map);
            }
        } else {
            // Random exploration when not carrying resources
            let mut rng = rand::thread_rng();
            let dx = rng.gen_range(-1..=1);
            let dy = rng.gen_range(-1..=1);

            let new_x = (self.x as isize + dx).clamp(0, map.config.width as isize - 1) as usize;
            let new_y = (self.y as isize + dy).clamp(0, map.config.height as isize - 1) as usize;

            if map.is_walkable(new_x, new_y) {
                self.x = new_x;
                self.y = new_y;
            }
        }
    }

    /// Définissez ici les comportements de collecte de ressources, etc.
    pub fn try_gather_resource(&mut self, map: &mut Map) -> bool {
        // Vérifions si on est dans les bornes
        if self.x >= map.config.width || self.y >= map.config.height {
            return false; 
        }

        match map.cells[self.y][self.x] {
            CellType::Energy => {
                // Le robot ne peut ramasser l'énergie que s'il a le module EnergyCollector
                if self.modules.contains(&RobotModule::EnergyCollector) {
                    self.carried_energy += 1;
                    // On supprime la ressource de la map
                    map.cells[self.y][self.x] = CellType::Empty;
                    return true;
                }
            }
            CellType::Mineral => {
                // Le robot doit avoir un module Drill pour miner
                if self.modules.contains(&RobotModule::Drill) {
                    self.carried_minerals += 1;
                    // On supprime la ressource
                    map.cells[self.y][self.x] = CellType::Empty;
                    return true;
                }
            }
            CellType::ScientificSite => {
                // On considère qu'on collecte des "données" du site.
                // Décidez si le site disparaît ou reste sur la map.
                // S'il est consommable = on le met à Empty.
                // Sinon, on pourrait le laisser.
                if self.modules.contains(&RobotModule::Exploration) {
                    self.carried_scientific_data += 1;
                    // On va supposer qu'il reste en place (ou qu'il est "visité")
                    // map.cells[self.y][self.x] = CellType::Empty; // si vous voulez le consommer
                    return true;
                }
            }
            _ => {}
        }

        false
    }

    /// Quand un robot revient à la station (maintenant au centre), on dépose les ressources
    pub fn try_deposit_resources(&mut self, station: &mut Station, map: &Map) {
        let center_x = map.config.width / 2;
        let center_y = map.config.height / 2;
        
        if self.is_near_base(center_x, center_y) {
            // Add the resources to the station's storage
            if self.carried_energy > 0 {
                station.add_energy(self.carried_energy);
                self.carried_energy = 0;
            }
            
            if self.carried_minerals > 0 {
                station.add_minerals(self.carried_minerals);
                self.carried_minerals = 0;
            }
            
            if self.carried_scientific_data > 0 {
                station.add_scientific_data(self.carried_scientific_data);
                self.carried_scientific_data = 0;
            }

            // Update the data sharing flag
            if self.has_data_to_share {
                self.has_data_to_share = false;
            }
        }
    }
}
