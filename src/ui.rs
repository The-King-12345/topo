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

    let mut pan_x = 0.0;
    let mut pan_y = 0.0;
    let pan_speed = 10.0;

    loop {
        terminal.draw(|f| {
            let area = f.area();
            
            let view_width = area.width as f64 * 2.0;
            let view_height = area.height as f64 * 4.0;
            let half_width = view_width / 2.0;
            let half_height = view_height / 2.0;

            let canvas = Canvas::default()
                .block(Block::bordered().title("Network Topology Map (HJKL to scroll)"))
                .x_bounds([pan_x - half_width, pan_x + half_width])
                .y_bounds([pan_y - half_height, pan_y + half_height])
                .paint(|ctx| {
                    ctx.layer();
                    
                    for (ip, host) in &network.hosts {
                        let box_width = 11.0;
                        let box_height = 11.0;

                        let bottom_left_x = host.x - (box_width / 2.0);
                        let bottom_left_y = host.y - (box_height / 2.0);

                        ctx.draw(&Rectangle {
                            x: bottom_left_x,
                            y: bottom_left_y,
                            width: box_width,
                            height: box_height,
                            color: Color::Green,
                        });

                        ctx.print(bottom_left_x, bottom_left_y - 9.0, host.host.clone());
                        ctx.print(bottom_left_x, bottom_left_y - 13.0, ip.clone());  
                    }
                });
            f.render_widget(canvas, area);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') | KeyCode::Char('C') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char('q') => break,
                    KeyCode::Char('h') => pan_x -= pan_speed, 
                    KeyCode::Char('l') => pan_x += pan_speed, 
                    KeyCode::Char('j') => pan_y -= pan_speed, 
                    KeyCode::Char('k') => pan_y += pan_speed, 
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}