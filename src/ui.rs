use crossterm::{
    event::{self, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Terminal,
};
use std::io;

use crate::network::Network;

fn apply_pan<'a>(lines: Vec<Line<'a>>, pan_x: i32, pan_y: i32) -> Vec<Line<'a>> {
    // Vertical pan
    let lines = if pan_y >= 0 {
        let skip = (pan_y as usize).min(lines.len());
        lines.into_iter().skip(skip).collect::<Vec<_>>()
    } else {
        let prepend = (-pan_y) as usize;
        let mut padded: Vec<Line> = (0..prepend).map(|_| Line::from("")).collect();
        padded.extend(lines);
        padded
    };

    // Horizontal pan
    lines
        .into_iter()
        .map(|line| {
            if pan_x == 0 {
                return line;
            }

            let mut new_spans: Vec<Span<'a>> = Vec::new();

            if pan_x > 0 {
                let mut to_skip = pan_x as usize;
                for span in line.spans {
                    let style = span.style;
                    let content = span.content.into_owned();
                    let char_count = content.chars().count();
                    if to_skip >= char_count {
                        to_skip = to_skip.saturating_sub(char_count);
                    } else {
                        let trimmed: String = content.chars().skip(to_skip).collect();
                        to_skip = 0;
                        new_spans.push(Span::styled(trimmed, style));
                    }
                }
            } else {
                let pad = (-pan_x) as usize;
                new_spans.push(Span::raw(" ".repeat(pad)));
                new_spans.extend(line.spans);
            }

            Line::from(new_spans)
        })
        .collect()
}

fn build_diagram<'a>(network: &Network) -> Vec<Line<'a>> {
    // Separate router (.1) from regular hosts
    let router_entry = network.hosts.iter().find(|(ip, _)| ip.ends_with(".1"));
    let hosts: Vec<(&String, &crate::network::HostData)> = network.hosts.iter()
        .filter(|(ip, _)| !ip.ends_with(".1"))
        .collect();
    let count = hosts.len();

    let cyan = Style::default().fg(Color::Cyan);
    let green = Style::default().fg(Color::Green);
    let gray = Style::default().fg(Color::DarkGray);

    let col_width = 20usize;
    let box_inner_width = col_width - 4; // accounts for "  │...│ " padding
    let inner = "─".repeat(box_inner_width);

    // How far from the left edge the switch should be centered
    let total_width = col_width * count;
    let switch_offset = if count > 0 { (total_width / 2).saturating_sub(8) } else { 0 };
    let switch_pad = " ".repeat(switch_offset);

    let mut lines: Vec<Line<'a>> = Vec::new();

    // ── ROUTER ──────────────────────────────────────────────
    let (router_ip_str, router_host_str) = match router_entry {
        Some((ip, data)) => (ip.as_str(), data.host.as_str()),
        None => ("?.?.?.1", "ROUTER"),
    };
    let router_host_label = format!("{:^14}", router_host_str);
    let router_ip_label   = format!("{:^14}", router_ip_str);

    lines.push(Line::from(vec![
        Span::raw(switch_pad.clone()),
        Span::styled("┌──────────────┐", cyan),
    ]));
    lines.push(Line::from(vec![
        Span::raw(switch_pad.clone()),
        Span::styled(format!("│{}│", router_host_label), cyan),
    ]));
    lines.push(Line::from(vec![
        Span::raw(switch_pad.clone()),
        Span::styled(format!("│{}│", router_ip_label), cyan),
    ]));
    lines.push(Line::from(vec![
        Span::raw(switch_pad.clone()),
        Span::styled("└──────────────┘", cyan),
    ]));

    // ── TRUNK WIRE ───────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::raw(switch_pad.clone()),
        Span::styled("       │        ", gray),
    ]));

    // ── HORIZONTAL BUS ───────────────────────────────────────
    if count == 0 {
        return lines;
    }

    let bus: String = (0..count)
        .map(|i| {
            let mut col: Vec<char> = " ".repeat(col_width).chars().collect();
            let center = col_width / 2;
            // First col: right half only; last col: left half only; middle: full
            for j in 0..col_width {
                let is_center = j == center;
                let in_bus = if i == 0 {
                    j >= center
                } else if i == count - 1 {
                    j <= center
                } else {
                    true
                };
                if in_bus && !is_center {
                    col[j] = '─';
                } else if is_center {
                    col[j] = if count > 1 { '┬' } else { '│' };
                }
            }
            col.iter().collect::<String>()
        })
        .collect();

    lines.push(Line::from(Span::styled(bus, gray)));

    // ── DROP WIRES ───────────────────────────────────────────
    let drop: String = hosts
        .iter()
        .map(|_| {
            let mut col: Vec<char> = " ".repeat(col_width).chars().collect();
            col[col_width / 2] = '│';
            col.iter().collect::<String>()
        })
        .collect();

    lines.push(Line::from(Span::styled(drop.clone(), gray)));
    lines.push(Line::from(Span::styled(drop, gray)));

    // ── HOST BOXES ───────────────────────────────────────────
    // Top border
    let top: String = hosts
        .iter()
        .map(|_| format!("  ┌{}┐ ", inner))
        .collect();
    lines.push(Line::from(Span::styled(top, green)));

    // Hostname row
    let name_row: Line = Line::from(
        hosts
            .iter()
            .map(|(_, host)| {
                let label = format!("{:^width$}", host.host, width = box_inner_width);
                let label = label[..label.len().min(box_inner_width)].to_string();
                Span::styled(format!("  │{}│ ", label), green)
            })
            .collect::<Vec<_>>(),
    );
    lines.push(name_row);

    // IP row (inside the box)
    let ip_inner_row: Line = Line::from(
        hosts
            .iter()
            .map(|(ip, _)| {
                let label = format!("{:^width$}", ip, width = box_inner_width);
                let label = label[..label.len().min(box_inner_width)].to_string();
                Span::styled(format!("  │{}│ ", label), green)
            })
            .collect::<Vec<_>>(),
    );
    lines.push(ip_inner_row);

    // Bottom border
    let bot: String = hosts
        .iter()
        .map(|_| format!("  └{}┘ ", inner))
        .collect();
    lines.push(Line::from(Span::styled(bot, green)));

    lines
}

pub fn draw_ui(network: &Network) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut pan_x: i32 = 0;
    let mut pan_y: i32 = 0;
    let pan_speed: i32 = 2;

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let lines = build_diagram(network);
            let lines = apply_pan(lines, pan_x, pan_y);

            let paragraph = Paragraph::new(lines).block(
                Block::bordered().title("Star Network Topology (HJKL to scroll, Q to quit)"),
            );

            f.render_widget(paragraph, area);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') | KeyCode::Char('C')
                        if key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        break;
                    }
                    KeyCode::Char('q') => break,
                    KeyCode::Char('h') => pan_x -= pan_speed,
                    KeyCode::Char('l') => pan_x += pan_speed,
                    KeyCode::Char('j') => pan_y += pan_speed,
                    KeyCode::Char('k') => pan_y -= pan_speed,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}