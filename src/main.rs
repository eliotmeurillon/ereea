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
        // Mise à jour des robots existants
        for robot in &mut sim.robots {
            // 1) Déplacement avec collision
            robot.random_move(&sim.map);

            // 2) Tenter de récupérer la ressource sur la case actuelle
            robot.try_gather_resource(&mut sim.map);

            // 3) Tenter de déposer à la station (maintenant au centre)
            robot.try_deposit_resources(&mut sim.station, &sim.map);
        }

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

