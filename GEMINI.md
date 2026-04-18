# Satellite Tracker TUI (orbitui) - Context

This project, `orbitui`, is a terminal-based satellite tracking application written in Rust. It uses SGP4 orbital propagation and live TLE data from Celestrak to provide real-time satellite positions across three interactive views: a 2D overhead map, a to-scale 3D globe, and an altitude-banded globe.

## Project Overview

- **Core Technologies:** Rust, `ratatui` (TUI), `crossterm` (Terminal handling), `tokio` (Async runtime), `sgp4` (Orbital mechanics), `reqwest` (HTTP), `serde`/`toml` (Config).
- **Architecture:** 
  - `src/main.rs`: Entry point and terminal setup.
  - `src/app.rs`: State machine and view routing.
  - `src/satellite/`: TLE fetching, parsing, and SGP4 propagation.
  - `src/views/`: Implementation of the three visualization modes.
  - `src/ui/`: Drawing primitives, projections, and widgets.

## Development Status

The project has completed **Phases 1-5** (Skeleton, Configuration, TLE Pipeline, Propagation, and Overhead Map). Real-time satellite tracking with spatial navigation is functional in the Overhead view. The `Project Plan.md` provides a detailed roadmap for the remaining phases.

## Building and Running

As a Rust project, the standard commands apply:

- **Build:** `cargo build`
- **Run:** `cargo run`
- **Test:** `cargo test`
- **Lint:** `cargo clippy`

## Key Files

- `Project Plan.md`: Comprehensive roadmap, architecture details, and implementation tasks.
- `README.md`: General overview, installation, and usage instructions.
- `config.toml`: User configuration for location and display preferences.
- `cache/`: Local TLE data cache.

## Development Conventions

- **Safety & Quality:** Use `cargo clippy` and `cargo fmt`.
- **Testing:** Unit tests are required for coordinate conversions and TLE parsing.
- **Async:** Use `tokio` for background tasks like TLE fetching and batch propagation to keep the UI responsive.
- **Coordination:** Adhere to the phases outlined in `Project Plan.md`.

## Interactive Controls

- `q` / `Ctrl+C`: Exit.
- `1`, `2`, `3`: Switch views.
- `Arrow Keys`: Spatial navigation (Overhead view).
- `+` / `-`: Filter satellite groups (Planned).
- `?`: Help overlay (Planned).
