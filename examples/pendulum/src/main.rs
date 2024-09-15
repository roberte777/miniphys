use crossterm::event::{self, Event, KeyCode};
use miniphys::pendulum::Pendulum;
use ratatui::style::Color;
use ratatui::widgets::canvas::Line;
use ratatui::widgets::{canvas::Canvas, Block, Borders};
use std::error::Error;
use std::time::{Duration, Instant};

struct App {
    pendulum: Pendulum,
    trail: Vec<(f64, f64)>,
}

impl App {
    fn new() -> Self {
        App {
            pendulum: Pendulum::new(1.0, 45.0, 0.1),
            trail: Vec::new(),
        }
    }

    fn update(&mut self, delta_time: f64) {
        self.pendulum.update(Duration::from_secs_f64(delta_time));
        let position = self.pendulum.position();
        self.trail.push(position);
        if self.trail.len() > 50 {
            self.trail.remove(0);
        }
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
                .title("Pendulum Simulation")
                .borders(Borders::ALL);
            f.render_widget(block, size);

            let canvas = Canvas::default()
                .x_bounds([-1.5, 1.5])
                .y_bounds([-1.5, 1.5])
                .paint(|ctx| {
                    // Draw the pendulum rod
                    let pen_pos = app.pendulum.position();
                    let line = Line::new(0.0, 0.0, pen_pos.0, pen_pos.1, Color::White);
                    ctx.draw(&line);

                    // Draw the pendulum bob
                    let (x, y) = app.pendulum.position();
                    ctx.print(x, y, "O");

                    // Draw the trail
                    for &(x, y) in &app.trail {
                        ctx.print(x, y, ".");
                    }
                });

            f.render_widget(canvas, size);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => {
                        // Reset pendulum on spacebar press
                        app.pendulum = Pendulum::new(1.0, 45.0, 0.1);
                        app.trail.clear();
                    }
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
