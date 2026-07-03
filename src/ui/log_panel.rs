use crate::log::SharedLog;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn draw_log_panel(f: &mut Frame, area: Rect, log: &SharedLog) {
    // Use a large portion of the screen (80% width, 70% height)
    let width = (area.width as f64 * 0.8) as u16;
    let height = (area.height as f64 * 0.7) as u16;
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;
    let panel_area = Rect::new(x, y, width, height);

    f.render_widget(Clear, panel_area);

    let block = Block::default()
        .title(" Event Log ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));

    let entries = log.entries();
    // Show the most recent entries that fit
    let max_lines = (height.saturating_sub(3)) as usize;
    let start = entries.len().saturating_sub(max_lines);
    let lines: Vec<Line> = entries[start..]
        .iter()
        .map(|(level, msg)| {
            let fg = match level.as_str() {
                "ERROR" => Color::Red,
                "WARN" => Color::Yellow,
                "INFO" => Color::Cyan,
                "DEBUG" => Color::Green,
                _ => Color::DarkGray,
            };
            Line::from(Span::styled(format!(" {}  {}", level, msg), Style::default().fg(fg)))
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, panel_area);
}