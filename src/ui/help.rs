use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn draw_help_overlay(f: &mut Frame, area: Rect) {
    // Compute a centered area (60% width, 60% height)
    let width = (area.width as f64 * 0.6) as u16;
    let height = (area.height as f64 * 0.6) as u16;
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;
    let help_area = Rect::new(x, y, width, height);

    // Clear the area behind the overlay
    f.render_widget(Clear, help_area);

    let block = Block::default()
        .title(" Help ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let lines = vec![
        Line::from("Controls:").style(Style::default().fg(Color::Cyan)),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("q / Ctrl+C", Style::default().fg(Color::Yellow)),
            Span::raw("    Exit"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("1 / 2 / 3", Style::default().fg(Color::Yellow)),
            Span::raw("    Switch views (Overhead / Sky / Bands)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Arrow keys", Style::default().fg(Color::Yellow)),
            Span::raw("  Navigate satellites (Overhead)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("+ / -", Style::default().fg(Color::Yellow)),
            Span::raw("      Zoom in / out (Overhead)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("?", Style::default().fg(Color::Yellow)),
            Span::raw("          Toggle this help"),
        ]),
        Line::from(""),
        Line::from("Satellite Colors:").style(Style::default().fg(Color::Cyan)),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("●", Style::default().fg(Color::Cyan)),
            Span::raw("  LEO    (< 2,000 km)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("●", Style::default().fg(Color::Yellow)),
            Span::raw("  MEO    (< 35,000 km)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("●", Style::default().fg(Color::Magenta)),
            Span::raw("  GEO    (~36,000 km)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("●", Style::default().fg(Color::Red)),
            Span::raw("  HEO    (> 37,000 km)"),
        ]),
        Line::from(""),
        Line::from("Dim gray dots indicate satellites below your horizon."),
        Line::from(""),
        Line::from("Press any key to close.").style(Style::default().fg(Color::DarkGray)),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, help_area);
}