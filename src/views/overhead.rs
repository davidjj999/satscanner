use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Map, MapResolution, Points},
        Block, Borders, Paragraph,
    },
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(30)].as_ref())
        .split(area);

    let map_area = chunks[0];
    let sidebar_area = chunks[1];

    let selected_idx = app.selected_overhead_idx;

    let mut map_points: Vec<(f64, f64)> = Vec::new();
    let mut leo_points: Vec<(f64, f64)> = Vec::new();
    let mut meo_points: Vec<(f64, f64)> = Vec::new();
    let mut geo_points: Vec<(f64, f64)> = Vec::new();
    let mut heo_points: Vec<(f64, f64)> = Vec::new();

    for (i, state) in app.sat_states.iter().enumerate() {
        // Skip the selected satellite — it is drawn as the ● highlight instead
        // to avoid sub-character misalignment between Points (braille) and print().
        if Some(i) == selected_idx { continue; }

        if state.el > 0.0 {
            if state.geodetic.alt < 2000.0 {
                leo_points.push((state.geodetic.lon, state.geodetic.lat));
            } else if state.geodetic.alt < 35000.0 {
                meo_points.push((state.geodetic.lon, state.geodetic.lat));
            } else if state.geodetic.alt < 37000.0 {
                geo_points.push((state.geodetic.lon, state.geodetic.lat));
            } else {
                heo_points.push((state.geodetic.lon, state.geodetic.lat));
            }
        } else {
            map_points.push((state.geodetic.lon, state.geodetic.lat));
        }
    }

    let obs_lat = app.config.lat;
    let obs_lon = app.config.lon;

    // Aspect ratio and zoom correction
    // terminal braille dots are 2x4 per character.
    let dot_width = map_area.width.saturating_sub(2) as f64 * 2.0; // Subtract borders
    let dot_height = map_area.height.saturating_sub(2) as f64 * 4.0;
    let r = if dot_height > 0.0 { dot_width / dot_height } else { 2.0 };

    // Set latitude span based on user's zoom level
    let y_span = app.zoom_level;
    
    // Scale X span based on terminal aspect ratio AND local latitude scaling (cos(lat))
    let cos_lat = obs_lat.to_radians().cos().max(0.1); 
    let x_span = (y_span * r) / cos_lat;

    let x_bounds = [obs_lon - x_span / 2.0, obs_lon + x_span / 2.0];
    let y_bounds = [obs_lat - y_span / 2.0, obs_lat + y_span / 2.0];

    let canvas = Canvas::default()
        .block(Block::default().title(" Overhead Map ").borders(Borders::ALL))
        .x_bounds(x_bounds)
        .y_bounds(y_bounds)
        .paint(|ctx| {
            ctx.draw(&Map {
                resolution: MapResolution::High,
                color: Color::Green,
            });

            ctx.draw(&Points {
                coords: &map_points,
                color: Color::DarkGray, // Dim dot for non-overhead
            });

            ctx.draw(&Points { coords: &leo_points, color: Color::Cyan });
            ctx.draw(&Points { coords: &meo_points, color: Color::Yellow });
            ctx.draw(&Points { coords: &geo_points, color: Color::Magenta });
            ctx.draw(&Points { coords: &heo_points, color: Color::Red });

            // Draw observer
            ctx.print(obs_lon, obs_lat, Span::styled("⊕", Style::default().fg(Color::White).bg(Color::Red).add_modifier(Modifier::BOLD)));

            // Highlight selected satellite
            if let Some(idx) = app.selected_overhead_idx
                && let Some(state) = app.sat_states.get(idx)
            {
                ctx.print(state.geodetic.lon, state.geodetic.lat, Span::styled("●", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
            }
        });

    f.render_widget(canvas, map_area);

    // Sidebar
    let mut sidebar_text = vec![];
    if let Some(idx) = app.selected_overhead_idx {
        if let Some(state) = app.sat_states.get(idx) {
            let sat_color = if state.geodetic.alt < 2000.0 { Color::Cyan }
                else if state.geodetic.alt < 35000.0 { Color::Yellow }
                else if state.geodetic.alt < 37000.0 { Color::Magenta }
                else { Color::Red };

            sidebar_text.push(Line::from(Span::styled(
                state.name.clone(),
                Style::default().fg(sat_color).add_modifier(Modifier::BOLD),
            )));
            sidebar_text.push(Line::from(""));
            sidebar_text.push(Line::from(format!("Alt: {:.1} km", state.geodetic.alt)));
            sidebar_text.push(Line::from(format!("Az:  {:.1}°", state.az)));
            sidebar_text.push(Line::from(format!("El:  {:.1}°", state.el)));
            sidebar_text.push(Line::from(format!("Rng: {:.1} km", state.range)));
            
            let vel_mag = (state.eci_vel[0].powi(2) + state.eci_vel[1].powi(2) + state.eci_vel[2].powi(2)).sqrt();
            sidebar_text.push(Line::from(format!("Vel: {:.2} km/s", vel_mag)));
        }
    } else {
        sidebar_text.push(Line::from(Span::styled("No satellite selected.", Style::default().fg(Color::DarkGray))));
        sidebar_text.push(Line::from(""));
        sidebar_text.push(Line::from("Use Right/Left arrow keys"));
        sidebar_text.push(Line::from("to cycle overhead satellites."));
    }

    let sidebar = Paragraph::new(sidebar_text)
        .block(Block::default().title(" Details ").borders(Borders::ALL));
    f.render_widget(sidebar, sidebar_area);
}
