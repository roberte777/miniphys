use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, MouseButton,
        MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Style},
    symbols,
    widgets::canvas::{Canvas, Circle, Line},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use miniphys::cloth::{Cloth, Vec2};

enum Event<I> {
    Input(I),
    Exit,
    Tick,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the cloth simulation
    let mut cloth = Cloth::new(10, 10, 5.0);
    for particle in cloth.particles() {
        println!(
            "particles: {}:{}",
            particle.position().x(),
            particle.position().y()
        );
    }

    // Setup terminal
    let mut terminal = ratatui::init();

    // Setup input handling
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(16); // ~60 FPS
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // Poll for events
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).unwrap() {
                if let CEvent::Mouse(mouse_event) = event::read().unwrap() {
                    tx.send(Event::Input(mouse_event)).unwrap();
                }
                if let crossterm::event::Event::Key(key) = event::read().unwrap() {
                    if let KeyCode::Char('q') = key.code {
                        let _ = tx.send(Event::Exit);
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    // Variables for interaction
    let mut dragging = false;
    let mut mouse_pos = Vec2::zero();
    let mut right_button = false;

    // Main loop
    loop {
        // Draw the UI
        terminal.draw(|f| ui(f, &cloth))?;

        // Handle input
        match rx.recv()? {
            Event::Exit => break,
            Event::Input(input) => match input {
                MouseEvent {
                    kind: MouseEventKind::Down(button),
                    column,
                    row,
                    ..
                } => {
                    println!("Got mouse event");
                    mouse_pos = Vec2::new(column as f64, row as f64);
                    if button == MouseButton::Right {
                        // Start dragging particles
                        dragging = true;
                        right_button = true;
                        cloth.select_particles(mouse_pos, 2.0);
                    } else if button == MouseButton::Left {
                        // Cut constraints at mouse position
                        cloth.cut_at_mouse(mouse_pos);
                    }
                }
                MouseEvent {
                    kind: MouseEventKind::Up(button),
                    ..
                } => {
                    if button == MouseButton::Right && right_button {
                        // Stop dragging particles
                        dragging = false;
                        right_button = false;
                        cloth.clear_selection();
                    }
                }
                MouseEvent {
                    kind: MouseEventKind::Moved,
                    column,
                    row,
                    ..
                } => {
                    mouse_pos = Vec2::new(column as f64, row as f64);
                    if dragging && right_button {
                        cloth.move_selected_particles(mouse_pos);
                    }
                }
                _ => {}
            },
            Event::Tick => {
                // Simulation step
                cloth.simulate(Duration::from_secs_f64(1.0 / 60.0));
            }
        }
    }

    // Cleanup terminal
    ratatui::restore();
    Ok(())
}

fn ui(f: &mut Frame, cloth: &Cloth) {
    let size = f.area();

    // Create a canvas widget to draw the cloth
    let canvas = Canvas::default()
        .block(ratatui::widgets::Block::default())
        .paint(|ctx| {
            // Draw constraints
            for constraint in cloth.constraints() {
                let (index_a, index_b) = constraint.particles();
                let p1 = cloth.particles()[index_a].position();
                let p2 = cloth.particles()[index_b].position();
                ctx.draw(&Line {
                    x1: p1.x(),
                    y1: p1.y(),
                    x2: p2.x(),
                    y2: p2.y(),
                    color: Color::White,
                });
            }

            // Draw particles
            for particle in cloth.particles() {
                let pos = particle.position();
                let color = if particle.pinned() {
                    Color::Red
                } else {
                    Color::Yellow
                };
                let circle = Circle {
                    x: pos.x(),
                    y: pos.y(),
                    radius: 0.5,
                    color,
                };
                ctx.draw(&circle);
            }
        })
        .x_bounds([0.0, size.width as f64])
        .y_bounds([0.0, size.height as f64]);

    f.render_widget(canvas, size);
}
