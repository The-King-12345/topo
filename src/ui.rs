use crossterm::{
    event::{self, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::Color,
    widgets::{
        canvas::{Canvas, Line, Rectangle},
        Block,
    },
    Terminal,
};
use std::io;

pub fn draw_ui() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let canvas = Canvas::default()
                .block(Block::bordered().title("Network Topology Map"))
                .x_bounds([-180.0, 180.0])
                .y_bounds([-90.0, 90.0])
                .paint(|ctx| {
                    ctx.layer();
                    ctx.draw(&Line { x1: 0.0, y1: 10.0, x2: 10.0, y2: 10.0, color: Color::White });
                    ctx.draw(&Rectangle { x: 10.0, y: 20.0, width: 10.0, height: 10.0, color: Color::Red });
                });
            f.render_widget(canvas, area);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') { break; }
                if ((key.code == KeyCode::Char('c') || key.code == KeyCode::char('C')) &&
                    key.modifiers.contains(KeyModifiers::CONTROL))
                {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}