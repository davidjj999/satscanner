# Satscanner

A terminal-based satellite tracking application written in Rust. It provides real-time satellite positions across interactive TUI views using SGP4 orbital propagation and live TLE data from Celestrak.

## Features

- **Real-time Tracking:** 60 FPS orbital propagation using SGP4.
- **Interactive Views:**
  - **Overhead Map:** 2D equirectangular map zoomed to your location with spatial navigation.
  - **Globe Views (Planned):** To-scale 3D globe and altitude-banded globe projections.
- **Live Data:** Automatically fetches and caches fresh TLE data from Celestrak.
- **Spatial Navigation:** Use arrow keys to select satellites based on their physical direction from your current selection.
- **Color-coded Regimes:** Satellites are categorized by altitude (LEO, MEO, GEO, HEO).

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- A terminal with Braille character support (most modern terminals).

### Building

```bash
cargo build --release
```

## Configuration

Create a `config.toml` in the project root to set your observer location:

```toml
lat = 37.7749
lon = -122.4194
alt = 10.0
location_name = "San Francisco"
```

If no config is found, it defaults to (0, 0).

## Usage

Run the application:

```bash
cargo run --release
```

### Controls

- `q` or `Ctrl+C`: Exit.
- `1`, `2`, `3`: Switch between views.
- `Arrow Keys`: Navigate between satellites (Overhead view).
- `?`: Help (Planned).

## Project Structure

- `src/main.rs`: Entry point and terminal lifecycle.
- `src/app.rs`: Main application state and logic.
- `src/satellite/`: TLE management and orbital mechanics.
- `src/views/`: Individual TUI view implementations.
- `src/ui/`: Shared UI components and projections.

## Roadmap

- [x] Phase 1: Skeleton & Event Loop
- [x] Phase 2: Configuration System
- [x] Phase 3: TLE Data Pipeline
- [x] Phase 4: Orbital Propagation Engine
- [x] Phase 5: Overhead Map View
- [ ] Phase 6: 3D Globe View (To-Scale)
- [ ] Phase 7: Altitude-Banded Globe View
- [ ] Phase 8: Performance Tuning & Polish

## License

MIT
