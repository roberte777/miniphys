use crossterm::event::{self, Event, KeyCode};
use miniphys::projectile_motion::Projectile;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{
        canvas::{Canvas, Line},
        Block, Borders, Paragraph,
    },
};
use std::error::Error;
use std::time::{Duration, Instant};

struct App {
    projectile: Projectile,
    trajectory: Vec<(f64, f64)>,
    time: f64,
}

impl App {
    fn new() -> Self {
        // Initial values for demonstration
        let initial_position = [0.0, 0.0];
        let initial_velocity = [10.0, 30.0]; // Adjust these values as needed
        let gravity = [0.0, -9.81]; // Gravity acts downward

        App {
            projectile: Projectile::new(initial_position, initial_velocity, gravity),
            trajectory: vec![initial_position.into()],
            time: 0.0,
        }
    }

    fn update(&mut self, delta_time: f64) {
        self.projectile.update(delta_time);
        self.trajectory.push(self.projectile.position());
        self.time += delta_time;

        // Reset if the projectile goes below the ground
        if self.projectile.position().1 < 0.0 {
            self.reset();
        }
    }

    fn reset(&mut self) {
        *self = App::new();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();

    let mut app = App::new();
    let tick_rate = Duration::from_millis(16); // ~60 FPS

    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| {
            let size = f.area();
            let block = Block::default()
                .title("Projectile Motion Simulation")
                .borders(Borders::ALL);
            f.render_widget(block, size);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
                .split(size);

            let canvas = Canvas::default()
                .x_bounds([0.0, 100.0])
                .y_bounds([0.0, 100.0])
                .paint(|ctx| {
                    // Draw the trajectory
                    for &(x, y) in &app.trajectory {
                        ctx.print(x, y, ".");
                    }

                    // Draw the current position of the projectile
                    let (x, y) = app.projectile.position();
                    ctx.print(x, y, "O");

                    // Draw the ground
                    let line = Line::new(0.0, 0.0, 100.0, 0.0, Color::Green);
                    ctx.draw(&line);
                });

            f.render_widget(canvas, chunks[0]);

            // Display time and position
            let (x, y) = app.projectile.position();
            let info = Paragraph::new(format!(
                "Time: {:.2}s | Position: ({:.2}, {:.2})",
                app.time, x, y
            ))
            .style(Style::default().fg(Color::White));
            f.render_widget(info, chunks[1]);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => app.reset(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.update(tick_rate.as_secs_f64());
            last_tick = Instant::now();
        }
    }

    ratatui::restore();

    Ok(())
}
