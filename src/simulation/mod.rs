use crate::environment::{Map, MapConfig};
use crate::robot::{Robot, RobotModule};
use crate::station::Station;

/// Une struct qui contient l'état global de la simulation
pub struct Simulation {
    pub map: Map,
    pub station: Station,
    pub robots: Vec<Robot>,
}

impl Simulation {
    pub fn new() -> Self {
        // Configuration par défaut
        let config = MapConfig {
            width: 50,
            height: 30,
            seed: 42,
        };
        let map = Map::new(config);
        let station = Station::new();

        // Create robots at the center of the map
        let mut robots = Vec::new();
        let center_x = map.config.width / 2;
        let center_y = map.config.height / 2;
        
        for i in 0..5 {
            robots.push(Robot::new(
                i,
                center_x,
                center_y,
                vec![
                    RobotModule::Exploration,
                    RobotModule::Drill,
                    RobotModule::EnergyCollector
                ]
            ));
        }

        Simulation {
            map,
            station,
            robots,
        }
    }
}

