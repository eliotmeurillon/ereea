use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::Span,
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
use crate::environment::map::CellType;

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

            // Draw status bar with enhanced styling
            let status_widget = Paragraph::new(status_text.clone())
                .block(Block::default()
                    .title(Span::styled("Status", Style::default().add_modifier(Modifier::BOLD)))
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)))
                .style(Style::default().bg(Color::Blue));
            frame.render_widget(status_widget, chunks[0]);

            // Draw map with enhanced styling
            let map_block = Block::default()
                .title(Span::styled("Map", Style::default().add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White));
            
            let inner_area = map_block.inner(chunks[1]);
            frame.render_widget(map_block, chunks[1]);

            // Create a canvas for the map
            let map_widget = Canvas::default()
                .paint(|ctx| {
                    // Adjust scaling to account for terminal cell aspect ratio
                    let cell_width = (inner_area.width as f64 - 2.0) / simulation.map.config.width as f64;
                    let cell_height = (inner_area.height as f64 - 1.0) / simulation.map.config.height as f64;  // Subtract 1 for bottom border

                    // Draw map cells
                    for y in 0..simulation.map.config.height {
                        for x in 0..simulation.map.config.width {
                            let (char, style) = match simulation.map.cells[y][x] {
                                CellType::Empty => ('.', Style::default().fg(Color::DarkGray)),
                                CellType::Obstacle => ('█', Style::default().fg(Color::Red)),
                                CellType::Energy => ('⚡', Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                                CellType::Mineral => ('◆', Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                                CellType::ScientificSite => ('✧', Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                            };
                            
                            let scaled_x = x as f64 * cell_width;
                            let scaled_y = y as f64 * cell_height;
                            
                            ctx.print(
                                scaled_x, 
                                scaled_y,
                                Span::styled(char.to_string(), style).to_string(),
                            );
                        }
                    }

                    // Draw robots with enhanced styling
                    for robot in &simulation.robots {
                        let scaled_x = robot.x as f64 * cell_width;
                        let scaled_y = robot.y as f64 * cell_height;
                        
                        ctx.print(
                            scaled_x,
                            scaled_y,
                            Span::styled(
                                "🤖",
                                Style::default()
                                    .fg(Color::LightGreen)
                                    .add_modifier(Modifier::BOLD)
                            ).to_string(),
                        );
                    }

                    // Draw the base station at the center with enhanced styling
                    let center_x = (simulation.map.config.width / 2) as f64 * cell_width;
                    let center_y = (simulation.map.config.height / 2) as f64 * cell_height;
                    
                    ctx.print(
                        center_x,
                        center_y,
                        Span::styled(
                            "🏠",
                            Style::default()
                                .fg(Color::White)
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