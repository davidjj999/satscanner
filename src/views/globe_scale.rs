use crate::app::App;
use crate::satellite::{coords::Geodetic, skypos};
use chrono::Utc;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Points},
        Block, Borders, Paragraph,
    },
    Frame,
};

/// Number of points used to approximate each elevation ring (horizon, 30°, 60°).
const RING_SEGMENTS: usize = 60;

/// Generate evenly-spaced points around a circle of the given radius.
fn ring_points(radius: f64) -> Vec<(f64, f64)> {
    (0..RING_SEGMENTS)
        .map(|i| {
            let a = 2.0 * std::f64::consts::PI * i as f64 / RING_SEGMENTS as f64;
            (radius * a.cos(), radius * a.sin())
        })
        .collect()
}

/// Convert azimuth (degrees, 0=N, 90=E) and elevation (degrees, 0=horizon, 90=zenith)
/// to canvas coordinates.  The sky circle has radius 1.0.
fn azel_to_xy(az_deg: f64, el_deg: f64) -> (f64, f64) {
    let r = 1.0 - el_deg / 90.0; // 0 at zenith, 1 at horizon
    let az_rad = az_deg.to_radians();
    // Azimuth 0° (North) → top of the circle, increasing clockwise
    (r * az_rad.sin(), r * az_rad.cos())
}

/// Satellite regime → colour (matches the overhead view).
fn regime_color(alt_km: f64) -> Color {
    if alt_km < 2000.0 {
        Color::Cyan
    } else if alt_km < 35000.0 {
        Color::Yellow
    } else if alt_km < 37000.0 {
        Color::Magenta
    } else {
        Color::Red
    }
}

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(30)].as_ref())
        .split(area);

    let sky_area = chunks[0];
    let sidebar_area = chunks[1];

    let now = Utc::now();
    let obs = Geodetic {
        lat: app.config.lat,
        lon: app.config.lon,
        alt: app.config.alt,
    };

    // --- Compute Sun and Moon positions ---
    let (sun_az, sun_el) = skypos::sun_position(&now, obs);
    let (moon_az, moon_el) = skypos::moon_position(&now, obs);

    let sun_visible = sun_el > 0.0;
    let moon_visible = moon_el > 0.0;

    let sun_xy = if sun_visible {
        Some(azel_to_xy(sun_az, sun_el))
    } else {
        None
    };

    let moon_xy = if moon_visible {
        Some(azel_to_xy(moon_az, moon_el))
    } else {
        None
    };

    // Build satellite point lists by regime, and track the selected satellite.
    let mut overhead_sats: Vec<(f64, f64, Color)> = Vec::new();
    let mut selected_pos: Option<(f64, f64)> = None;

    for (i, state) in app.sat_states.iter().enumerate() {
        if state.el <= 0.0 {
            continue; // skip below-horizon satellites in sky view
        }
        let (x, y) = azel_to_xy(state.az, state.el);

        if Some(i) == app.selected_overhead_idx {
            selected_pos = Some((x, y));
            continue; // don't also draw the coloured dot underneath
        }

        let color = regime_color(state.geodetic.alt);
        overhead_sats.push((x, y, color));
    }

    let canvas = Canvas::default()
        .block(Block::default().title(" Sky View ").borders(Borders::ALL))
        .x_bounds([-1.3, 1.3])
        .y_bounds([-1.3, 1.3])
        .paint(|ctx| {
            // --- Elevation rings ---
            let ring_colors = [
                (1.0, Color::DarkGray),       // horizon  (0°)
                (2.0 / 3.0, Color::DarkGray), // 30°
                (1.0 / 3.0, Color::DarkGray), // 60°
            ];

            for (radius, color) in &ring_colors {
                ctx.draw(&Points {
                    coords: &ring_points(*radius),
                    color: *color,
                });
            }

            // --- Cardinal direction labels ---
            ctx.print(0.0, 1.05, Span::styled("N", Style::default().fg(Color::White)));
            ctx.print(1.05, 0.0, Span::styled("E", Style::default().fg(Color::White)));
            ctx.print(0.0, -1.10, Span::styled("S", Style::default().fg(Color::White)));
            ctx.print(-1.10, 0.0, Span::styled("W", Style::default().fg(Color::White)));

            // --- Elevation labels ---
            ctx.print(0.05, 0.68, Span::styled("60°", Style::default().fg(Color::DarkGray)));
            ctx.print(0.05, 0.35, Span::styled("30°", Style::default().fg(Color::DarkGray)));

            // --- Sun (if above horizon) ---
            if let Some((sx, sy)) = sun_xy {
                // Draw a bright circle behind the label for visibility
                ctx.draw(&Points {
                    coords: &[(sx, sy)],
                    color: Color::Yellow,
                });
                ctx.print(
                    sx - 0.055,
                    sy + 0.02,
                    Span::styled(
                        "☀",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                );
                ctx.print(
                    sx + 0.065,
                    sy + 0.02,
                    Span::styled(
                        "SUN",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                );
            }

            // --- Moon (if above horizon) ---
            if let Some((mx, my)) = moon_xy {
                ctx.draw(&Points {
                    coords: &[(mx, my)],
                    color: Color::White,
                });
                ctx.print(
                    mx - 0.055,
                    my + 0.02,
                    Span::styled(
                        "☽",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                );
                ctx.print(
                    mx + 0.065,
                    my + 0.02,
                    Span::styled(
                        "MOON",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                );
            }

            // --- Satellite dots by regime ---
            // Draw each colour group as a single Points call for efficiency.
            for color in [Color::Cyan, Color::Yellow, Color::Magenta, Color::Red] {
                let coords: Vec<(f64, f64)> = overhead_sats
                    .iter()
                    .filter(|(_, _, c)| *c == color)
                    .map(|(x, y, _)| (*x, *y))
                    .collect();
                if !coords.is_empty() {
                    ctx.draw(&Points {
                        coords: &coords,
                        color,
                    });
                }
            }

            // --- Observer at zenith ---
            ctx.print(
                0.0,
                0.0,
                Span::styled(
                    "⊕",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
            );

            // --- Highlight selected satellite ---
            if let Some((x, y)) = selected_pos {
                ctx.print(
                    x,
                    y,
                    Span::styled(
                        "●",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                );
            }
        });

    f.render_widget(canvas, sky_area);

    // --- Sidebar (reuses the same layout as overhead) ---
    let mut sidebar_text = vec![];
    if let Some(idx) = app.selected_overhead_idx {
        if let Some(state) = app.sat_states.get(idx) {
            let sat_color = regime_color(state.geodetic.alt);

            sidebar_text.push(Line::from(Span::styled(
                state.name.clone(),
                Style::default().fg(sat_color).add_modifier(Modifier::BOLD),
            )));
            sidebar_text.push(Line::from(""));
            sidebar_text.push(Line::from(format!("Alt: {:.1} km", state.geodetic.alt)));
            sidebar_text.push(Line::from(format!("Az:  {:.1}°", state.az)));
            sidebar_text.push(Line::from(format!("El:  {:.1}°", state.el)));
            sidebar_text.push(Line::from(format!("Rng: {:.1} km", state.range)));

            let vel_mag = (state.eci_vel[0].powi(2)
                + state.eci_vel[1].powi(2)
                + state.eci_vel[2].powi(2))
            .sqrt();
            sidebar_text.push(Line::from(format!("Vel: {:.2} km/s", vel_mag)));
        }
    } else {
        sidebar_text.push(Line::from(Span::styled(
            "No satellite selected.",
            Style::default().fg(Color::DarkGray),
        )));
        sidebar_text.push(Line::from(""));
        sidebar_text.push(Line::from("Use arrow keys"));
        sidebar_text.push(Line::from("to select a satellite."));
    }

    let sidebar = Paragraph::new(sidebar_text)
        .block(Block::default().title(" Details ").borders(Borders::ALL));
    f.render_widget(sidebar, sidebar_area);
}