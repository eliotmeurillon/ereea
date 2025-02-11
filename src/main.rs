use env_logger;
use log::info;

mod environment;
mod robot;
mod station;
mod simulation;
mod ui;
mod pathfinding;

use std::{io, thread, time::Duration};

fn main() -> Result<(), io::Error> {
    env_logger::init();
    info!("Lancement de la simulation EREEA...");

    let mut ui = ui::Ui::new()?;
    let mut sim = simulation::Simulation::new();

    for _step in 0..300 {
        // Effacer la visibilité précédente
        sim.map.fade_visibility();

        // Mise à jour des robots existants
        for robot in &mut sim.robots {
            // Mettre à jour la visibilité autour du robot
            sim.map.update_visibility(robot.x, robot.y, 2);  // Rayon de vision de 2

            robot.random_move(&sim.map);
            robot.try_gather_resource(&mut sim.map);
            robot.try_deposit_resources(&mut sim.station, &sim.map);
        }

        // Toujours garder la base visible
        sim.map.update_visibility(sim.map.config.width / 2, sim.map.config.height / 2, 3);

        // Vérifier si on peut créer un nouveau robot
        if let Some(new_robot) = sim.station.try_create_robot() {
            info!("Nouveau robot créé! ID: {}", new_robot.id);
            sim.robots.push(new_robot);
        }

        // Affichage
        ui.draw(&sim)?;
        thread::sleep(Duration::from_millis(100));
    }

    info!("Fin de la simulation.");
    Ok(())
}

