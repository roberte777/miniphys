use crossterm::event::{self, Event, KeyCode};
use miniphys::spring::Spring;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use std::error::Error;
use std::time::{Duration, Instant};

// Include your Spring struct and implementation here
// ... (copy the Spring struct and implementation from the provided code)

struct App {
    spring: Spring,
    position: f64,
    velocity: f64,
    target: f64,
    width: u16,
}

impl App {
    fn new(width: u16) -> Self {
        let spring = Spring::new(0.0, 0.0, 6.0, 1.2);
        App {
            spring,
            position: 0.0,
            velocity: 0.0,
            target: width as f64 / 2.0,
            width,
        }
    }

    fn update(&mut self) {
        (self.position, self.velocity) = self.spring.update(Duration::from_secs(16), self.target);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let mut app = App::new(terminal.size()?.width);
    let tick_rate = Duration::from_millis(16); // ~60 FPS

    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(size);

            let block = Block::default().title("Spring Demo").borders(Borders::ALL);
            f.render_widget(block, chunks[0]);

            let square_pos = app.position.round() as u16;

            let target = Paragraph::new("X")
                .style(Style::default().fg(Color::Red))
                .alignment(ratatui::layout::Alignment::Left);
            f.render_widget(
                target,
                ratatui::layout::Rect::new(app.target as u16, chunks[0].y + 1, 1, 1),
            );

            let square = Paragraph::new("■")
                .style(Style::default().fg(Color::Yellow))
                .alignment(ratatui::layout::Alignment::Left);
            f.render_widget(
                square,
                ratatui::layout::Rect::new(square_pos, chunks[0].y + 1, 1, 1),
            );

            let stats_block = Block::default().title("Stats").borders(Borders::ALL);
            let text = vec![
                Line::from(format!("Square Pos: {}", app.position)),
                Line::from(format!("Target Pos: {}", app.target)),
            ];
            let stats = Paragraph::new(text).block(stats_block);
            f.render_widget(stats, chunks[1]);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    break;
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.update();
            last_tick = Instant::now();
        }
    }

    ratatui::restore();

    Ok(())
}
