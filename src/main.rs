use env_logger;
use log::info;

mod environment;
mod pathfinding;
mod robot;
mod simulation;
mod station;
mod ui;

use crossterm::event::{self, Event, KeyCode};
use std::{io, time::Duration};

fn main() -> Result<(), io::Error> {
    env_logger::init();
    info!("Starting EREEA simulation...");

    let mut ui = ui::Ui::new()?;
    let mut sim = simulation::Simulation::new();

    let max_steps = 1000;

    loop {
        sim.update();

        ui.draw(&sim)?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        info!("User requested exit. Terminating simulation.");
                        break;
                    }
                    _ => {}
                }
            }
        }

        if sim.stats.simulation_step >= max_steps {
            info!(
                "Reached maximum simulation steps ({}). Terminating.",
                max_steps
            );
            break;
        }

        if sim.stats.simulation_step % 100 == 0 {
            info!("Simulation step: {}", sim.stats.simulation_step);
            info!("Robots: {}", sim.robots.len());
            info!(
                "Resources collected - Energy: {}, Minerals: {}, Scientific Data: {}",
                sim.stats.total_energy_collected,
                sim.stats.total_minerals_collected,
                sim.stats.total_scientific_data_collected
            );
        }
    }

    info!("Simulation complete. Final statistics:");
    info!("Total steps: {}", sim.stats.simulation_step);
    info!("Robots created: {}", sim.stats.robots_created);
    info!(
        "Resources collected - Energy: {}, Minerals: {}, Scientific Data: {}",
        sim.stats.total_energy_collected,
        sim.stats.total_minerals_collected,
        sim.stats.total_scientific_data_collected
    );

    Ok(())
}
