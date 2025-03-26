use crate::robot::{Robot, RobotModule};

#[derive(Debug)]
pub struct Station {
    pub energy_storage: u32,
    pub minerals_storage: u32,
    pub scientific_data_count: u32,
    robot_counter: usize,
    explorer_count: usize,
    driller_count: usize,
    energy_collector_count: usize,
}

impl Station {
    pub fn new() -> Self {
        Self {
            energy_storage: 0,
            minerals_storage: 0,
            scientific_data_count: 0,
            robot_counter: 5,
            explorer_count: 2,
            driller_count: 2,
            energy_collector_count: 1,
        }
    }

    pub fn add_energy(&mut self, amount: u32) {
        self.energy_storage += amount;
    }

    pub fn add_minerals(&mut self, amount: u32) {
        self.minerals_storage += amount;
    }

    pub fn add_scientific_data(&mut self, amount: u32) {
        self.scientific_data_count += amount;
    }

    pub fn update_robot_counts(&mut self, robot_type: &RobotModule) {
        match robot_type {
            RobotModule::Exploration => self.explorer_count += 1,
            RobotModule::Drill => self.driller_count += 1,
            RobotModule::EnergyCollector => self.energy_collector_count += 1,
        }
    }

    pub fn try_create_robot(&mut self) -> Option<Robot> {
        let min_resources_needed = 1;

        if self.energy_storage >= min_resources_needed
            && self.minerals_storage >= min_resources_needed
            && self.scientific_data_count >= min_resources_needed
        {
            let robot_module = self.determine_next_robot_type();

            let resource_cost = match &robot_module {
                RobotModule::Exploration => 1,
                RobotModule::Drill => 1,
                RobotModule::EnergyCollector => 1,
            };

            self.energy_storage -= resource_cost;
            self.minerals_storage -= resource_cost;
            self.scientific_data_count -= resource_cost;

            let robot = Robot::new(
                self.robot_counter,
                self.get_center_x(),
                self.get_center_y(),
                vec![robot_module.clone()],
            );

            self.robot_counter += 1;
            self.update_robot_counts(&robot_module);

            Some(robot)
        } else {
            None
        }
    }

    fn determine_next_robot_type(&self) -> RobotModule {
        let total_robots = self.explorer_count + self.driller_count + self.energy_collector_count;

        let explorer_percent = self.explorer_count as f32 / total_robots as f32;
        let driller_percent = self.driller_count as f32 / total_robots as f32;
        let energy_collector_percent = self.energy_collector_count as f32 / total_robots as f32;

        const TARGET_EXPLORER_PERCENT: f32 = 0.4;
        const TARGET_DRILLER_PERCENT: f32 = 0.3;
        const TARGET_ENERGY_PERCENT: f32 = 0.3;

        let explorer_deficit = TARGET_EXPLORER_PERCENT - explorer_percent;
        let driller_deficit = TARGET_DRILLER_PERCENT - driller_percent;
        let energy_deficit = TARGET_ENERGY_PERCENT - energy_collector_percent;

        let resource_adjusted_driller_deficit = if self.minerals_storage < 5 {
            driller_deficit + 0.2
        } else {
            driller_deficit
        };

        let resource_adjusted_energy_deficit = if self.energy_storage < 5 {
            energy_deficit + 0.2
        } else {
            energy_deficit
        };

        let resource_adjusted_explorer_deficit = if self.scientific_data_count < 5 {
            explorer_deficit + 0.2
        } else {
            explorer_deficit
        };

        if resource_adjusted_explorer_deficit > resource_adjusted_driller_deficit
            && resource_adjusted_explorer_deficit > resource_adjusted_energy_deficit
        {
            RobotModule::Exploration
        } else if resource_adjusted_driller_deficit > resource_adjusted_energy_deficit {
            RobotModule::Drill
        } else {
            RobotModule::EnergyCollector
        }
    }

    fn get_center_x(&self) -> usize {
        50 / 2
    }

    fn get_center_y(&self) -> usize {
        30 / 2
    }
}
