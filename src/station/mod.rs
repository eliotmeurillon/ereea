use crate::robot::{Robot, RobotModule};

/// Représente la station de base sur la planète
#[derive(Debug)]
pub struct Station {
    pub energy_storage: u32,
    pub minerals_storage: u32,
    pub scientific_data_count: u32,
    robot_counter: usize,  // Pour générer des IDs uniques
}

impl Station {
    pub fn new() -> Self {
        Self {
            energy_storage: 0,
            minerals_storage: 0,
            scientific_data_count: 0,
            robot_counter: 5,  // Commencer à 5 car nous avons déjà 5 robots initiaux
        }
    }

    /// Ajoute de l'énergie au stockage
    pub fn add_energy(&mut self, amount: u32) {
        self.energy_storage += amount;
    }

    /// Ajoute des minéraux au stockage
    pub fn add_minerals(&mut self, amount: u32) {
        self.minerals_storage += amount;
    }

    /// Ajoute des données scientifiques
    pub fn add_scientific_data(&mut self, amount: u32) {
        self.scientific_data_count += amount;
    }

    pub fn try_create_robot(&mut self) -> Option<Robot> {
        // Vérifier si nous avons assez de ressources
        if self.energy_storage >= 1 && 
           self.minerals_storage >= 1 && 
           self.scientific_data_count >= 1 {
            
            // Consommer les ressources
            self.energy_storage -= 1;
            self.minerals_storage -= 1;
            self.scientific_data_count -= 1;

            // Créer un nouveau robot avec tous les modules
            let robot = Robot::new(
                self.robot_counter,
                self.get_center_x(),
                self.get_center_y(),
                vec![
                    RobotModule::Exploration,
                    RobotModule::Drill,
                    RobotModule::EnergyCollector,
                ]
            );

            self.robot_counter += 1;
            Some(robot)
        } else {
            None
        }
    }

    fn get_center_x(&self) -> usize {
        50 / 2  // Largeur de la carte / 2
    }

    fn get_center_y(&self) -> usize {
        30 / 2  // Hauteur de la carte / 2
    }
}
