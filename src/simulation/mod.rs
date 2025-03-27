use crate::environment::{Map, MapConfig};
use crate::robot::{Robot, RobotModule};
use crate::station::Station;
use crossbeam::channel::{unbounded, Receiver, Sender};
use log::info;

#[derive(Debug, Clone)]
pub enum SimulationEvent {
    ResourceCollected {
        resource_type: ResourceType,
        amount: u32,
    },
    RobotCreated {
        id: usize,
    },
}

#[derive(Debug, Clone)]
pub enum ResourceType {
    Energy,
    Mineral,
    ScientificData,
}

pub struct Simulation {
    pub map: Map,
    pub station: Station,
    pub robots: Vec<Robot>,
    event_sender: Option<Sender<SimulationEvent>>,
    event_receiver: Option<Receiver<SimulationEvent>>,

    pub stats: SimulationStats,
}

#[derive(Debug, Default)]
pub struct SimulationStats {
    pub total_energy_collected: u32,
    pub total_minerals_collected: u32,
    pub total_scientific_data_collected: u32,
    pub robots_created: u32,
    pub simulation_step: usize,
}

impl Simulation {
    pub fn new() -> Self {
        let config = MapConfig {
            width: 50,
            height: 30,
            seed: 42,
        };
        let map = Map::new(config);
        let station = Station::new();

        let (sender, receiver) = unbounded();

        let mut robots = Vec::new();
        let center_x = map.config.width / 2;
        let center_y = map.config.height / 2;

        for i in 0..2 {
            robots.push(Robot::new(
                i,
                center_x,
                center_y,
                vec![RobotModule::Exploration],
            ));
        }

        for i in 2..4 {
            robots.push(Robot::new(i, center_x, center_y, vec![RobotModule::Drill]));
        }

        robots.push(Robot::new(
            4,
            center_x,
            center_y,
            vec![RobotModule::EnergyCollector],
        ));

        Simulation {
            map,
            station,
            robots,
            event_sender: Some(sender),
            event_receiver: Some(receiver),
            stats: SimulationStats::default(),
        }
    }

    pub fn update(&mut self) {
        self.map.fade_visibility();

        for i in 0..self.robots.len() {
            let robot = &self.robots[i];
            self.map.update_visibility(robot.x, robot.y, 2);

            let mut specialized_move = false;

            if self.robots[i].should_return_to_base() {
                let center_x = self.map.config.width / 2;
                let center_y = self.map.config.height / 2;

                if !self.robots[i].is_near_base(center_x, center_y) {
                    self.robots[i].move_towards(center_x, center_y, &self.map);
                    specialized_move = true;
                }
            } else {
                if self.robots[i].modules.contains(&RobotModule::Exploration) {
                    if let Some((target_x, target_y)) =
                        find_unexplored_area(self.robots[i].x, self.robots[i].y, &self.map)
                    {
                        self.robots[i].move_towards(target_x, target_y, &self.map);
                        specialized_move = true;
                    }
                } else if self.robots[i].modules.contains(&RobotModule::Drill) {
                    if let Some((target_x, target_y)) = find_nearest_resource(
                        self.robots[i].x,
                        self.robots[i].y,
                        &self.map,
                        crate::environment::map::CellType::Mineral,
                    ) {
                        self.robots[i].move_towards(target_x, target_y, &self.map);
                        specialized_move = true;
                    }
                } else if self.robots[i]
                    .modules
                    .contains(&RobotModule::EnergyCollector)
                {
                    if let Some((target_x, target_y)) = find_nearest_resource(
                        self.robots[i].x,
                        self.robots[i].y,
                        &self.map,
                        crate::environment::map::CellType::Energy,
                    ) {
                        self.robots[i].move_towards(target_x, target_y, &self.map);
                        specialized_move = true;
                    }
                }
            }

            if !specialized_move {
                self.robots[i].random_move(&self.map);
            }

            if self.robots[i].try_gather_resource(&mut self.map) {
                let resource_type = if self.robots[i].carried_energy > 0 {
                    ResourceType::Energy
                } else if self.robots[i].carried_minerals > 0 {
                    ResourceType::Mineral
                } else {
                    ResourceType::ScientificData
                };

                if let Some(ref sender) = self.event_sender {
                    let _ = sender.send(SimulationEvent::ResourceCollected {
                        resource_type,
                        amount: 1,
                    });
                }
            }
        }

        self.map
            .update_visibility(self.map.config.width / 2, self.map.config.height / 2, 3);

        for robot in &mut self.robots {
            robot.try_deposit_resources(&mut self.station, &self.map);
        }

        if let Some(new_robot) = self.station.try_create_robot() {
            let robot_id = new_robot.id;
            if let Some(ref sender) = self.event_sender {
                let _ = sender.send(SimulationEvent::RobotCreated { id: robot_id });
            }
            self.robots.push(new_robot);
            self.stats.robots_created += 1;
            info!("Created new robot with ID: {}", robot_id);
        }

        self.process_events();

        self.stats.simulation_step += 1;
    }

    fn process_events(&mut self) {
        if let Some(ref receiver) = self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    SimulationEvent::ResourceCollected {
                        resource_type,
                        amount,
                    } => match resource_type {
                        ResourceType::Energy => {
                            self.stats.total_energy_collected += amount;
                        }
                        ResourceType::Mineral => {
                            self.stats.total_minerals_collected += amount;
                        }
                        ResourceType::ScientificData => {
                            self.stats.total_scientific_data_collected += amount;
                        }
                    },
                    SimulationEvent::RobotCreated { id } => {
                        info!("Processed robot creation event for robot ID: {}", id);
                    }
                }
            }
        }
    }
}

fn find_unexplored_area(robot_x: usize, robot_y: usize, map: &Map) -> Option<(usize, usize)> {
    use crate::environment::map::CellVisibility;

    let mut radius = 3;
    let max_radius = 10;

    while radius <= max_radius {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx * dx + dy * dy >= (radius - 2) * (radius - 2)
                    && dx * dx + dy * dy <= radius * radius
                {
                    let x =
                        (robot_x as isize + dx).clamp(0, map.config.width as isize - 1) as usize;
                    let y =
                        (robot_y as isize + dy).clamp(0, map.config.height as isize - 1) as usize;

                    if map.visibility[y][x] == CellVisibility::Hidden && map.is_walkable(x, y) {
                        return Some((x, y));
                    }
                }
            }
        }

        radius += 3;
    }

    radius = 3;
    while radius <= max_radius {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx * dx + dy * dy >= (radius - 2) * (radius - 2)
                    && dx * dx + dy * dy <= radius * radius
                {
                    let x =
                        (robot_x as isize + dx).clamp(0, map.config.width as isize - 1) as usize;
                    let y =
                        (robot_y as isize + dy).clamp(0, map.config.height as isize - 1) as usize;

                    if map.visibility[y][x] == CellVisibility::Explored && map.is_walkable(x, y) {
                        return Some((x, y));
                    }
                }
            }
        }

        radius += 3;
    }

    None
}

fn find_nearest_resource(
    robot_x: usize,
    robot_y: usize,
    map: &Map,
    resource_type: crate::environment::map::CellType,
) -> Option<(usize, usize)> {
    use crate::environment::map::CellVisibility;

    let mut closest_dist = f32::MAX;
    let mut closest_point = None;

    for y in 0..map.config.height {
        for x in 0..map.config.width {
            if map.visibility[y][x] != CellVisibility::Hidden && map.cells[y][x] == resource_type {
                let dist = ((x as isize - robot_x as isize).pow(2)
                    + (y as isize - robot_y as isize).pow(2)) as f32;

                if dist < closest_dist {
                    closest_dist = dist;
                    closest_point = Some((x, y));
                }
            }
        }
    }

    closest_point
}
