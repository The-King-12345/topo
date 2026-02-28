use crossterm::{
    event::{self, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::Color,
    widgets::{
        canvas::{Canvas, Rectangle},
        Block,
    },
    Terminal,
};
use std::io;

use crate::network::Network;

pub fn draw_ui(network: &Network) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let usable_height = area.height.saturating_sub(2).max(1) as f64;
            let units_per_row = 180.0 / usable_height;
            let canvas = Canvas::default()
                .block(Block::bordered().title("Network Topology Map"))
                .x_bounds([-180.0, 180.0])
                .y_bounds([-90.0, 90.0])
                .paint(|ctx| {
                    ctx.layer();
                    
                    for (ip, host) in &network.hosts {
                        ctx.draw(&Rectangle {
                            x: host.x,
                            y: host.y,
                            width: 10.0,
                            height: 10.0,
                            color: Color::Green,
                        });

                        ctx.print(host.x, host.y - (units_per_row * 2.0), host.host.clone());
                        ctx.print(host.x, host.y - (units_per_row * 3.5), ip.clone());  
                    }
                });
            f.render_widget(canvas, area);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') { break; }
                if (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('C')) &&
                    key.modifiers.contains(KeyModifiers::CONTROL)
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