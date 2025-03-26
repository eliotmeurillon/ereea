use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{canvas::Canvas, Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use std::io;

use crate::environment::map::{CellType, CellVisibility};
use crate::robot::RobotModule;
use crate::simulation::Simulation;

pub struct Ui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Ui {
    pub fn new() -> Result<Self, io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    pub fn draw(&mut self, simulation: &Simulation) -> Result<(), io::Error> {
        let status_text = format!(
            "Energy: {} | Minerals: {} | Data: {} | Robots: {} | Step: {}",
            simulation.station.energy_storage,
            simulation.station.minerals_storage,
            simulation.station.scientific_data_count,
            simulation.robots.len(),
            simulation.stats.simulation_step
        );

        self.terminal.draw(|frame| {
            // Create main layout with status bar at top, map and details panels below
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(frame.size());

            // Create horizontal layout for map and details panel
            let content_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
                .split(main_layout[1]);

            // Render status bar
            let status_widget = Paragraph::new(Line::from(vec![Span::styled(
                status_text,
                Style::default().fg(Color::White),
            )]))
            .block(
                Block::default()
                    .title(Span::styled(
                        "Status",
                        Style::default().add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            )
            .style(Style::default().bg(Color::Indexed(17)));
            frame.render_widget(status_widget, main_layout[0]);

            // Render map
            let map_block = Block::default()
                .title(Span::styled(
                    "Map",
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL);

            let inner_area = map_block.inner(content_layout[0]);
            frame.render_widget(map_block, content_layout[0]);

            let map_widget = Canvas::default()
                .paint(|ctx| {
                    let cell_width =
                        (inner_area.width as f64 - 2.0) / simulation.map.config.width as f64;
                    let cell_height =
                        (inner_area.height as f64 - 1.0) / simulation.map.config.height as f64;

                    // Draw map cells
                    for y in 0..simulation.map.config.height {
                        for x in 0..simulation.map.config.width {
                            let (char, mut style) = match simulation.map.cells[y][x] {
                                CellType::Empty => ('.', Style::default().fg(Color::Indexed(245))),
                                CellType::Obstacle => {
                                    ('#', Style::default().fg(Color::Rgb(100, 40, 40)))
                                }
                                CellType::Energy => (
                                    '⚡',
                                    Style::default()
                                        .fg(Color::Indexed(226))
                                        .add_modifier(Modifier::BOLD),
                                ),
                                CellType::Mineral => (
                                    '💎',
                                    Style::default()
                                        .fg(Color::Indexed(51))
                                        .add_modifier(Modifier::BOLD),
                                ),
                                CellType::ScientificSite => (
                                    '🔬',
                                    Style::default()
                                        .fg(Color::Indexed(201))
                                        .add_modifier(Modifier::BOLD),
                                ),
                            };

                            match simulation.map.visibility[y][x] {
                                CellVisibility::Hidden => {
                                    style = Style::default()
                                        .fg(Color::Rgb(0, 0, 0))
                                        .bg(Color::Rgb(0, 0, 0));
                                    ctx.print(
                                        x as f64 * cell_width,
                                        y as f64 * cell_height,
                                        Span::styled(" ", style).to_string(),
                                    );
                                }
                                CellVisibility::Explored => {
                                    style = style.fg(Color::Rgb(40, 40, 40));
                                    ctx.print(
                                        x as f64 * cell_width,
                                        y as f64 * cell_height,
                                        Span::styled(char.to_string(), style).to_string(),
                                    );
                                }
                                CellVisibility::Visible => {
                                    ctx.print(
                                        x as f64 * cell_width,
                                        y as f64 * cell_height,
                                        Span::styled(char.to_string(), style).to_string(),
                                    );
                                }
                            }
                        }
                    }

                    // Draw the base station (draw it before robots to ensure robots are on top)
                    let center_x = (simulation.map.config.width / 2) as f64 * cell_width;
                    let center_y = (simulation.map.config.height / 2) as f64 * cell_height;

                    ctx.print(
                        center_x,
                        center_y,
                        Span::styled(
                            "🏠",
                            Style::default()
                                .fg(Color::Indexed(231))
                                .add_modifier(Modifier::BOLD),
                        )
                        .to_string(),
                    );

                    // Draw robots with different visuals based on their module
                    for robot in &simulation.robots {
                        let scaled_x = robot.x as f64 * cell_width;
                        let scaled_y = robot.y as f64 * cell_height;

                        // Different visual representation based on robot module
                        let (robot_char, robot_color) =
                            if robot.modules.contains(&RobotModule::Exploration) {
                                ("🔍", Color::Indexed(86)) // Explorer robots - magnifying glass in cyan
                            } else if robot.modules.contains(&RobotModule::Drill) {
                                ("⛏️", Color::Indexed(214)) // Drill robots - pickaxe in orange
                            } else if robot.modules.contains(&RobotModule::EnergyCollector) {
                                ("🔋", Color::Indexed(118)) // Energy collector - battery in green
                            } else {
                                ("🤖", Color::Indexed(250)) // Generic robot in white
                            };

                        // Add small indicator if robot is carrying resources
                        let carrying = robot.carried_energy > 0
                            || robot.carried_minerals > 0
                            || robot.carried_scientific_data > 0;
                        let robot_style =
                            Style::default().fg(robot_color).add_modifier(if carrying {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            });

                        ctx.print(
                            scaled_x,
                            scaled_y,
                            Span::styled(robot_char, robot_style).to_string(),
                        );
                    }
                })
                .x_bounds([0.0, inner_area.width as f64])
                .y_bounds([0.0, inner_area.height as f64]);

            frame.render_widget(map_widget, inner_area);

            let details_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(7),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ]
                    .as_ref(),
                )
                .split(content_layout[1]);

            // Render legend block
            let legend_block = Block::default()
                .title(Span::styled(
                    "Legend",
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL);

            let legend_text = Text::from(vec![
                Line::from(vec![
                    Span::styled("🔍 ", Style::default().fg(Color::Indexed(86))),
                    Span::raw("Explorer"),
                ]),
                Line::from(vec![
                    Span::styled("⛏️ ", Style::default().fg(Color::Indexed(214))),
                    Span::raw("Miner"),
                ]),
                Line::from(vec![
                    Span::styled("🔋 ", Style::default().fg(Color::Indexed(118))),
                    Span::raw("Energy Collector"),
                ]),
                Line::from(vec![
                    Span::styled("⚡ ", Style::default().fg(Color::Indexed(226))),
                    Span::raw("Energy"),
                ]),
                Line::from(vec![
                    Span::styled("💎 ", Style::default().fg(Color::Indexed(51))),
                    Span::raw("Mineral"),
                ]),
            ]);

            let legend_widget = Paragraph::new(legend_text)
                .block(legend_block)
                .wrap(Wrap { trim: true });

            frame.render_widget(legend_widget, details_layout[0]);

            let stats_text = format!(
                "Energy: {} | Minerals: {} | Science: {}",
                simulation.stats.total_energy_collected,
                simulation.stats.total_minerals_collected,
                simulation.stats.total_scientific_data_collected,
            );

            let stats_block = Paragraph::new(stats_text).block(
                Block::default()
                    .title(Span::styled(
                        "Resources Collected",
                        Style::default().add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            );

            frame.render_widget(stats_block, details_layout[1]);

            let mut robot_items = Vec::new();
            let mut explorer_count = 0;
            let mut miner_count = 0;
            let mut energy_count = 0;

            for robot in &simulation.robots {
                if robot.modules.contains(&RobotModule::Exploration) {
                    explorer_count += 1;
                } else if robot.modules.contains(&RobotModule::Drill) {
                    miner_count += 1;
                } else if robot.modules.contains(&RobotModule::EnergyCollector) {
                    energy_count += 1;
                }
            }

            robot_items.push(ListItem::new(format!("Explorers: {}", explorer_count)));
            robot_items.push(ListItem::new(format!("Miners: {}", miner_count)));
            robot_items.push(ListItem::new(format!(
                "Energy Collectors: {}",
                energy_count
            )));

            if !simulation.robots.is_empty() {
                robot_items.push(ListItem::new(""));
                robot_items.push(ListItem::new("Active robots:"));

                let max_visible_robots = if details_layout[2].height > 10 {
                    (details_layout[2].height as usize - 6).min(simulation.robots.len())
                } else {
                    3.min(simulation.robots.len())
                };

                for (_i, robot) in simulation
                    .robots
                    .iter()
                    .enumerate()
                    .take(max_visible_robots)
                {
                    let robot_type = if robot.modules.contains(&RobotModule::Exploration) {
                        "Explorer"
                    } else if robot.modules.contains(&RobotModule::Drill) {
                        "Miner"
                    } else if robot.modules.contains(&RobotModule::EnergyCollector) {
                        "Energy"
                    } else {
                        "Unknown"
                    };

                    let carrying = if robot.carried_energy > 0 {
                        format!("⚡{}", robot.carried_energy)
                    } else if robot.carried_minerals > 0 {
                        format!("💎{}", robot.carried_minerals)
                    } else if robot.carried_scientific_data > 0 {
                        format!("🔬{}", robot.carried_scientific_data)
                    } else {
                        "".to_string()
                    };

                    robot_items.push(ListItem::new(format!(
                        "#{}: {} {}",
                        robot.id, robot_type, carrying
                    )));
                }

                if simulation.robots.len() > max_visible_robots {
                    robot_items.push(ListItem::new(format!(
                        "... {} more",
                        simulation.robots.len() - max_visible_robots
                    )));
                }
            }

            let robot_list = List::new(robot_items).block(
                Block::default()
                    .title(Span::styled(
                        "Robots",
                        Style::default().add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            );

            frame.render_widget(robot_list, details_layout[2]);
        })?;

        Ok(())
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
    }
}
