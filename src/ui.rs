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
use std::collections::HashMap;
use std::io;

use crate::network::Network;

const BOX_W: f64 = 28.0;
const BOX_H: f64 = 10.0;
const LEVEL_HEIGHT: f64 = 50.0;
const H_SPACING: f64 = 36.0;

struct NodeLayout {
    label: String,
    x: f64,
    y: f64,
    parent: Option<String>,
}

fn build_layout(network: &Network) -> Vec<NodeLayout> {
    let mut children: HashMap<String, Vec<String>> = HashMap::new();
    let mut roots: Vec<String> = Vec::new();

    for (name, host) in &network.hosts {
        if host.parent.is_empty() {
            roots.push(name.clone());
        } else {
            children
                .entry(host.parent.clone())
                .or_default()
                .push(name.clone());
        }
    }

    for v in children.values_mut() {
        v.sort();
    }
    roots.sort();

    let mut layout: Vec<NodeLayout> = Vec::new();
    // Single global cursor: each leaf gets the next slot
    let mut leaf_x: f64 = 0.0;

    fn place(
        name: &str,
        parent: Option<&str>,
        depth: usize,
        children: &HashMap<String, Vec<String>>,
        leaf_x: &mut f64,
        layout: &mut Vec<NodeLayout>,
    ) {
        let kids = children.get(name).map(|v| v.as_slice()).unwrap_or(&[]);

        if kids.is_empty() {
            let cx = *leaf_x;
            *leaf_x += H_SPACING;
            layout.push(NodeLayout {
                label: name.to_string(),
                x: cx,
                y: -(depth as f64 * LEVEL_HEIGHT),
                parent: parent.map(|s| s.to_string()),
            });
        } else {
            let start_idx = layout.len();
            for kid in kids.to_vec() {
                place(&kid, Some(name), depth + 1, children, leaf_x, layout);
            }
            // Center parent over its children
            let child_xs: Vec<f64> = layout[start_idx..].iter().map(|n| n.x).collect();
            let min_cx = child_xs.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_cx = child_xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let cx = (min_cx + max_cx) / 2.0;
            layout.push(NodeLayout {
                label: name.to_string(),
                x: cx,
                y: -(depth as f64 * LEVEL_HEIGHT),
                parent: parent.map(|s| s.to_string()),
            });
        }
    }

    for root in &roots {
        place(root, None, 0, &children, &mut leaf_x, &mut layout);
    }

    let min_x = layout.iter().map(|n| n.x).fold(f64::INFINITY, f64::min);
    for n in &mut layout {
        n.x -= min_x;
    }

    layout
}

pub fn draw_ui(network: &Network) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let layout = build_layout(network);

    // Center pan on the diagram
    let cx = layout.iter().map(|n| n.x).fold(f64::INFINITY, f64::min)
        + (layout.iter().map(|n| n.x).fold(f64::NEG_INFINITY, f64::max)
            - layout.iter().map(|n| n.x).fold(f64::INFINITY, f64::min))
            / 2.0;
    let cy = layout.iter().map(|n| n.y).fold(f64::INFINITY, f64::min)
        + (layout.iter().map(|n| n.y).fold(f64::NEG_INFINITY, f64::max)
            - layout.iter().map(|n| n.y).fold(f64::INFINITY, f64::min))
            / 2.0;

    let mut pan_x = cx;
    let mut pan_y = cy;
    let pan_speed = 10.0;

    // Build position lookup for drawing lines
    let pos: HashMap<String, (f64, f64)> = layout
        .iter()
        .map(|n| (n.label.clone(), (n.x, n.y)))
        .collect();

    loop {
        terminal.draw(|f| {
            let area = f.area();

            let inner_width = area.width.saturating_sub(2).max(1) as f64;
            let inner_height = area.height.saturating_sub(2).max(1) as f64;

            let view_width = inner_width * 2.0;
            let view_height = inner_height * 4.0;

            let half_width = view_width / 2.0;
            let half_height = view_height / 2.0;

            let canvas = Canvas::default()
                .block(Block::bordered().title("Network Topology (HJKL to scroll, Q to quit)"))
                .x_bounds([pan_x - half_width, pan_x + half_width])
                .y_bounds([pan_y - half_height, pan_y + half_height])
                .paint(|ctx| {
                    ctx.layer();

                    for node in &layout {
                        if let Some(ref p) = node.parent {
                            if let Some(&(px, py)) = pos.get(p) {
                                let y1 = py - BOX_H / 2.0;      // parent bottom
                                let y3 = node.y + BOX_H / 2.0;  // child top

                                // straight lines
                                let y2 = (y1 + y3) / 2.0;       // elbow row
                                // ctx.draw(&Line { x1: px, y1, x2: px, y2, color: Color::Gray });
                                // ctx.draw(&Line { x1: px, y1: y2, x2: node.x, y2, color: Color::Gray });
                                // ctx.draw(&Line { x1: node.x, y1: y2, x2: node.x, y2: y3, color: Color::Gray });


                                ctx.draw(&Line {x1: px, y1, x2: node.x, y2: y3, color: Color::Gray});
                            }
                        }
                    }

                    // Draw boxes and labels
                    for node in &layout {
                        let bx = node.x - BOX_W / 2.0;
                        let by = node.y - BOX_H / 2.0;

                        ctx.draw(&Rectangle {
                            x: bx,
                            y: by,
                            width: BOX_W,
                            height: BOX_H,
                            color: Color::Green,
                        });

                        // Truncate label to fit box
                        let max_chars = ((BOX_W - 2.0) as usize).max(1);
                        let display = if node.label.len() > max_chars {
                            &node.label[..max_chars]
                        } else {
                            &node.label
                        };
                        ctx.print(bx + 2.0, node.y - 1.5, display.to_string());
                    }
                });

            f.render_widget(canvas, area);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') | KeyCode::Char('C')
                        if key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        break
                    }
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