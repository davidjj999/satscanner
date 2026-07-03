# Satscanner

A terminal-based satellite tracking application written in Rust. It provides real-time satellite positions across interactive TUI views using SGP4 orbital propagation and live TLE data from Celestrak.

## Features

- **Real-time Tracking:** 60 FPS orbital propagation using SGP4.
- **Interactive Views:**
  - **Overhead Map:** 2D equirectangular map zoomed to your location with spatial navigation.
  - **Sky View:** Polar plot of your local sky showing satellites at their true azimuth and elevation. Includes ☀ Sun and ☽ Moon positions, labeled space stations, and elevation rings.
  - **Orbital Cross-Section (Planned):** Side-on altitude view replacing the altitude-banded globe.
- **Live Data:** Automatically fetches and caches fresh TLE data from Celestrak.
- **Screen-space Navigation:** Arrow keys always select the nearest satellite in the direction they point on screen, regardless of view projection.
- **Color-coded Regimes:** Satellites categorized by altitude (LEO, MEO, GEO, HEO). Space stations highlighted in **light green** with labels.
- **Help Overlay:** Press `?` for a full keybind reference.
- **Debug Logging:** Press `l` to view live logs; also written to a rolling `satscanner.log` file (1 MB rotation).

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- A terminal with Braille character support (most modern terminals).

### Building

```bash
cargo build --release
```

## Configuration

Copy the example config and edit it with your location:

```bash
cp config.toml.example config.toml
```

Then edit `config.toml`:

```toml
lat = 37.7749
lon = -122.4194
alt = 10.0
location_name = "San Francisco"
```

If no config is found, it defaults to (0, 0) — Null Island.

## Usage

Run the application:

```bash
cargo run --release
```

## Visual Conventions

### Overhead Map Symbols
- **Green Lines:** World coastlines.
- **⊕ (White on Red):** Your configured observer location.
- **● (White Highlight):** Currently selected satellite.
- **◆ (LightGreen):** Crewed space station (ISS / CSS), with label.
- **Dim Gray Dots:** Satellites currently below your horizon.

### Sky View Symbols
- **⊕ (White on Red):** Your location at zenith.
- **☀ SUN (Yellow):** The Sun's position (when above horizon).
- **☽ MOON (White):** The Moon's position (when above horizon).
- **◆ (LightGreen) + label:** Crewed space station at its true az/el.
- **Elevation rings:** Horizon (outer), 30°, 60°.
- **Cardinal labels:** N, E, S, W at the sky circle edge.

### Satellite Color Coding (both views)
Satellites above your horizon are color-coded by orbital regime:
- **LightGreen (◆):** Crewed space stations (ISS, Tiangong/CSS)
- **Cyan:** LEO (Low Earth Orbit, < 2,000 km)
- **Yellow:** MEO (Medium Earth Orbit, < 35,000 km)
- **Magenta:** GEO (Geostationary Orbit, ~36,000 km)
- **Red:** HEO (High Earth Orbit / Highly Elliptical)

### Controls

| Key | Action |
|---|---|
| `q`, `Esc`, `Ctrl+C` | Exit |
| `1` / `2` / `3` | Switch views (Overhead / Sky / Bands) |
| `Arrow Keys` | Navigate satellites by screen position (all views) |
| `+` / `-` | Zoom in / zoom out (Overhead view) |
| `?` | Toggle help overlay |
| `l` | Toggle in-app event log viewer |

### Debug Logging

Logs are written to `satscanner.log` in the project root. The file auto-rotates at 1 MB, keeping one backup (`satscanner.log.1`). You can also view logs live in the app by pressing `l`.

For verbose output (propagation counts per tick, etc.), set the env var before launching:

```bash
RUST_LOG=satscanner=trace cargo run --release
```

## Project Structure

```
satscanner/
├── Cargo.toml
├── config.toml              # User location (git-ignored)
├── config.toml.example      # Example config file
├── satscanner.log           # Rolling debug log (git-ignored)
├── cache/                   # Local TLE data cache
├── src/
│   ├── main.rs              # Entry point and terminal lifecycle
│   ├── app.rs               # Main application state and logic
│   ├── config.rs            # Config loading and defaults
│   ├── log.rs               # In-memory ring buffer + rolling file log
│   ├── satellite/
│   │   ├── mod.rs
│   │   ├── coords.rs        # ECI → ECEF → Geodetic conversions
│   │   ├── propagate.rs     # SGP4 wrapper, batch position compute
│   │   ├── skypos.rs        # Sun and Moon position algorithms
│   │   └── tle.rs           # TLE fetch, parse, cache
│   ├── views/
│   │   ├── mod.rs           # View enum (Overhead, Sky, GlobeBands)
│   │   ├── overhead.rs      # 2D equirectangular map
│   │   ├── globe_scale.rs   # Sky View (polar az/el plot)
│   │   └── globe_bands.rs   # Orbital cross-section (stub)
│   └── ui/
│       ├── mod.rs           # Top-level draw routing
│       ├── widgets.rs       # Status bar, info sidebar
│       ├── help.rs          # Help overlay popup
│       └── log_panel.rs     # In-app log viewer overlay
```

## Roadmap

- [x] Phase 1–2: Skeleton, Configuration
- [x] Phase 3: TLE Data Pipeline
- [x] Phase 4: Orbital Propagation Engine
- [x] Phase 5: Overhead Map View
- [x] Phase 6: Sky View (polar plot with Sun/Moon, space stations)
- [ ] Phase 7: Orbital Cross-Section View (altitude-centric)
- [x] Phase 8: Polish — Help overlay, config.toml.example, rolling log, log viewer, space station highlighting, screen-space navigation
- [ ] Phase 8: Performance Tuning, CI, packaging
- [ ] Phase 9: Packaging & Documentation

## License

MIT