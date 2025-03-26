use crate::environment::map::CellType;
use crate::environment::Map;
use crate::pathfinding;
use crate::station::Station;
use rand::Rng;

#[derive(Debug, PartialEq, Clone)]
pub enum RobotModule {
    Exploration,
    Drill,
    EnergyCollector,
}

#[derive(Debug)]
pub struct Robot {
    pub id: usize,
    pub x: usize,
    pub y: usize,
    pub modules: Vec<RobotModule>,
    pub has_data_to_share: bool,

    pub carried_energy: u32,
    pub carried_minerals: u32,
    pub carried_scientific_data: u32,
    last_dx: i32,
    last_dy: i32,
}

impl Robot {
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
            last_dx: 0,
            last_dy: 0,
        }
    }

    pub fn should_return_to_base(&self) -> bool {
        self.carried_energy > 0 || self.carried_minerals > 0 || self.carried_scientific_data > 0
    }

    pub fn move_towards(&mut self, target_x: usize, target_y: usize, map: &Map) {
        let start = (self.x, self.y);
        let goal = (target_x, target_y);

        if let Some(path) = pathfinding::find_path(map, start, goal) {
            if path.len() > 1 {
                let next_pos = path[1];
                self.x = next_pos.0;
                self.y = next_pos.1;
            }
        } else {
            let dx = if self.x < target_x {
                1
            } else if self.x > target_x {
                -1
            } else {
                0
            };
            let dy = if self.y < target_y {
                1
            } else if self.y > target_y {
                -1
            } else {
                0
            };

            let new_x = (self.x as isize + dx).clamp(0, map.config.width as isize - 1) as usize;
            let new_y = (self.y as isize + dy).clamp(0, map.config.height as isize - 1) as usize;

            if map.is_walkable(new_x, new_y) {
                self.x = new_x;
                self.y = new_y;
            }
        }
    }

    pub fn is_near_base(&self, center_x: usize, center_y: usize) -> bool {
        let dx = self.x.abs_diff(center_x);
        let dy = self.y.abs_diff(center_y);
        dx <= 1 && dy <= 1
    }

    pub fn random_move(&mut self, map: &Map) {
        let center_x = map.config.width / 2;
        let center_y = map.config.height / 2;

        if self.should_return_to_base() {
            if !self.is_near_base(center_x, center_y) {
                self.move_towards(center_x, center_y, map);
            }
        } else {
            let mut rng = rand::thread_rng();

            if rng.gen_bool(0.8) && (self.last_dx != 0 || self.last_dy != 0) {
                let new_x =
                    (self.x as i32 + self.last_dx).clamp(0, map.config.width as i32 - 1) as usize;
                let new_y =
                    (self.y as i32 + self.last_dy).clamp(0, map.config.height as i32 - 1) as usize;

                if map.is_walkable(new_x, new_y) {
                    self.x = new_x;
                    self.y = new_y;
                    return;
                }
            }

            let dx = rng.gen_range(-1..=1);
            let dy = rng.gen_range(-1..=1);

            let new_x = (self.x as i32 + dx).clamp(0, map.config.width as i32 - 1) as usize;
            let new_y = (self.y as i32 + dy).clamp(0, map.config.height as i32 - 1) as usize;

            if map.is_walkable(new_x, new_y) {
                self.x = new_x;
                self.y = new_y;
                self.last_dx = dx;
                self.last_dy = dy;
            }
        }
    }

    pub fn try_gather_resource(&mut self, map: &mut Map) -> bool {
        if self.x >= map.config.width || self.y >= map.config.height {
            return false;
        }

        match map.cells[self.y][self.x] {
            CellType::Energy => {
                if self.modules.contains(&RobotModule::EnergyCollector) {
                    self.carried_energy += 1;
                    map.cells[self.y][self.x] = CellType::Empty;
                    return true;
                }
            }
            CellType::Mineral => {
                if self.modules.contains(&RobotModule::Drill) {
                    self.carried_minerals += 1;
                    map.cells[self.y][self.x] = CellType::Empty;
                    return true;
                }
            }
            CellType::ScientificSite => {
                if self.modules.contains(&RobotModule::Exploration) {
                    self.carried_scientific_data += 1;
                    return true;
                }
            }
            _ => {}
        }

        false
    }

    pub fn try_deposit_resources(&mut self, station: &mut Station, map: &Map) {
        let center_x = map.config.width / 2;
        let center_y = map.config.height / 2;

        if self.is_near_base(center_x, center_y) {
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

            if self.has_data_to_share {
                self.has_data_to_share = false;
            }
        }
    }
}
