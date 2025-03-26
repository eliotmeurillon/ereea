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

// A struct to represent how a cell should be displayed
#[derive(Clone)]
enum CellDisplay {
    Char(char, Style),
    Str(&'static str, Style),
}

impl CellDisplay {
    fn is_empty(&self) -> bool {
        match self {
            CellDisplay::Char(c, _) => *c == ' ',
            CellDisplay::Str(s, _) => s.is_empty(),
        }
    }

    fn to_styled_string(&self) -> String {
        match self {
            CellDisplay::Char(c, style) => Span::styled(c.to_string(), *style).to_string(),
            CellDisplay::Str(s, style) => Span::styled(s.to_string(), *style).to_string(),
        }
    }
}

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
                    // Use fixed spacing for cells
                    // Set a fixed spacing that works well in terminal
                    let cell_spacing_x = 2.0; // Horizontal spacing between cells
                    let cell_spacing_y = 1.0; // Vertical spacing between cells
                    
                    // Calculate total grid dimensions
                    let grid_width = (simulation.map.config.width as f64 - 1.0) * cell_spacing_x;
                    let grid_height = (simulation.map.config.height as f64 - 1.0) * cell_spacing_y;
                    
                    // Calculate offsets to center the grid in the available area
                    let offset_x = (inner_area.width as f64 - grid_width) / 2.0;
                    let offset_y = (inner_area.height as f64 - grid_height) / 2.0;

                    // Draw map cells
                    for y in 0..simulation.map.config.height {
                        for x in 0..simulation.map.config.width {
                            // Get the appropriate character and style based on cell type and visibility
                            let cell_display = match simulation.map.visibility[y][x] {
                                CellVisibility::Hidden => {
                                    // Colored fog for hidden areas - using full block character to fill the entire cell
                                    CellDisplay::Str("██", Style::default().fg(Color::Rgb(30, 30, 50)).bg(Color::Rgb(10, 10, 20)))
                                }
                                CellVisibility::Explored => {
                                    match simulation.map.cells[y][x] {
                                        CellType::Empty => CellDisplay::Char(' ', Style::default()), // Transparent floor
                                        CellType::Obstacle => CellDisplay::Str("🏔️", Style::default().fg(Color::Rgb(80, 80, 80))), // Faded mountain emoji
                                        CellType::Energy => CellDisplay::Char('⚡', Style::default().fg(Color::Rgb(80, 80, 0))),
                                        CellType::Mineral => CellDisplay::Char('💎', Style::default().fg(Color::Rgb(20, 50, 50))),
                                        CellType::ScientificSite => CellDisplay::Char('🔬', Style::default().fg(Color::Rgb(80, 40, 80))),
                                    }
                                }
                                CellVisibility::Visible => {
                                    match simulation.map.cells[y][x] {
                                        CellType::Empty => CellDisplay::Char(' ', Style::default()), // Transparent floor
                                        CellType::Obstacle => CellDisplay::Str("🏔️", Style::default().fg(Color::Rgb(160, 120, 90))), // Mountain emoji
                                        CellType::Energy => CellDisplay::Char('⚡', Style::default().fg(Color::Indexed(226)).add_modifier(Modifier::BOLD)),
                                        CellType::Mineral => CellDisplay::Char('💎', Style::default().fg(Color::Indexed(51)).add_modifier(Modifier::BOLD)),
                                        CellType::ScientificSite => CellDisplay::Char('🔬', Style::default().fg(Color::Indexed(201)).add_modifier(Modifier::BOLD)),
                                    }
                                }
                            };

                            // Calculate position using fixed spacing
                            let pos_x = offset_x + (x as f64 * cell_spacing_x);
                            let pos_y = offset_y + (y as f64 * cell_spacing_y);

                            // Only print if there's something to display
                            if !cell_display.is_empty() {
                                ctx.print(
                                    pos_x,
                                    pos_y,
                                    cell_display.to_styled_string(),
                                );
                            }
                        }
                    }

                    // Draw the base station with the same positioning logic
                    let center_x = offset_x + ((simulation.map.config.width / 2) as f64 * cell_spacing_x);
                    let center_y = offset_y + ((simulation.map.config.height / 2) as f64 * cell_spacing_y);

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
                        let scaled_x = offset_x + (robot.x as f64 * cell_spacing_x);
                        let scaled_y = offset_y + (robot.y as f64 * cell_spacing_y);

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
