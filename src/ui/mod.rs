use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Modifier, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, canvas::Canvas},
    Terminal,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

use crate::simulation::Simulation;
use crate::environment::map::{CellType, CellVisibility};

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
            "Energy: {} | Minerals: {} | Data: {}", 
            simulation.station.energy_storage,
            simulation.station.minerals_storage,
            simulation.station.scientific_data_count
        );

        self.terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Status bar
                    Constraint::Min(0),     // Map area
                ].as_ref())
                .split(frame.size());

            // Status bar
            let status_widget = Paragraph::new(Line::from(vec![
                Span::styled(status_text, Style::default().fg(Color::White))
            ]))
            .block(Block::default()
                .title(Span::styled("Status", Style::default().add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL))
            .style(Style::default().bg(Color::Indexed(17)));  // Dark blue background
            frame.render_widget(status_widget, chunks[0]);

            // Map area
            let map_block = Block::default()
                .title(Span::styled("Map", Style::default().add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL);
            
            let inner_area = map_block.inner(chunks[1]);
            frame.render_widget(map_block, chunks[1]);

            // Map content
            let map_widget = Canvas::default()
                .paint(|ctx| {
                    let cell_width = (inner_area.width as f64 - 2.0) / simulation.map.config.width as f64;
                    let cell_height = (inner_area.height as f64 - 1.0) / simulation.map.config.height as f64;

                    // Draw map cells
                    for y in 0..simulation.map.config.height {
                        for x in 0..simulation.map.config.width {
                            let (char, mut style) = match simulation.map.cells[y][x] {
                                CellType::Empty => ('.', Style::default().fg(Color::Indexed(245))),  // Light gray
                                CellType::Obstacle => ('#', Style::default().fg(Color::Rgb(100, 40, 40))),  // Dark reddish brown
                                CellType::Energy => ('⚡', Style::default().fg(Color::Indexed(226)).add_modifier(Modifier::BOLD)),
                                CellType::Mineral => ('💎', Style::default().fg(Color::Indexed(51)).add_modifier(Modifier::BOLD)),
                                CellType::ScientificSite => ('🔬', Style::default().fg(Color::Indexed(201)).add_modifier(Modifier::BOLD)),
                            };

                            match simulation.map.visibility[y][x] {
                                CellVisibility::Hidden => {
                                    style = Style::default()
                                        .fg(Color::Rgb(0, 0, 0))  // Pure black foreground
                                        .bg(Color::Rgb(0, 0, 0)); // Pure black background
                                    ctx.print(
                                        x as f64 * cell_width,
                                        y as f64 * cell_height,
                                        Span::styled(" ", style).to_string(),
                                    );
                                },
                                CellVisibility::Explored => {
                                    style = style.fg(Color::Rgb(40, 40, 40));  // Dark gray
                                    ctx.print(
                                        x as f64 * cell_width,
                                        y as f64 * cell_height,
                                        Span::styled(char.to_string(), style).to_string(),
                                    );
                                },
                                CellVisibility::Visible => {
                                    ctx.print(
                                        x as f64 * cell_width,
                                        y as f64 * cell_height,
                                        Span::styled(char.to_string(), style).to_string(),
                                    );
                                },
                            }
                        }
                    }

                    // Draw robots
                    for robot in &simulation.robots {
                        let scaled_x = robot.x as f64 * cell_width;
                        let scaled_y = robot.y as f64 * cell_height;
                        
                        ctx.print(
                            scaled_x,
                            scaled_y,
                            Span::styled(
                                "🤖",
                                Style::default()
                                    .fg(Color::Indexed(46))  // Bright green
                                    .add_modifier(Modifier::BOLD)
                            ).to_string(),
                        );
                    }

                    // Draw base
                    let center_x = (simulation.map.config.width / 2) as f64 * cell_width;
                    let center_y = (simulation.map.config.height / 2) as f64 * cell_height;
                    
                    ctx.print(
                        center_x,
                        center_y,
                        Span::styled(
                            "🏠",
                            Style::default()
                                .fg(Color::Indexed(231))  // Pure white
                                .add_modifier(Modifier::BOLD)
                        ).to_string(),
                    );
                })
                .x_bounds([0.0, inner_area.width as f64])
                .y_bounds([0.0, inner_area.height as f64]);

            frame.render_widget(map_widget, inner_area);
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
        ).unwrap();
    }
}