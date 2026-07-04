# Satellite Tracker TUI (satscanner) — Context

This project, `satscanner`, is a terminal-based satellite tracking application written in Rust. It uses SGP4 orbital propagation and live TLE data from Celestrak to provide real-time satellite positions across three interactive views: a 2D overhead map, a polar sky view, and an orbital cross-section (planned).

## Project Overview

- **Core Technologies:** Rust, `ratatui` (TUI), `crossterm` (Terminal handling), `tokio` (Async runtime), `sgp4` (Orbital mechanics), `reqwest` (HTTP), `serde`/`toml` (Config).
- **Architecture:** 
  - `src/main.rs`: Entry point, terminal setup, and event loop.
  - `src/app.rs`: State machine, view routing, and tick logic.
  - `src/log.rs`: In-memory ring buffer + rolling file log, wired to `tracing`.
  - `src/config.rs`: Config loading from `config.toml` with defaults.
  - `src/satellite/`: TLE fetching, parsing, SGP4 propagation, coordinate transforms, and Sun/Moon ephemeris.
  - `src/views/`: Implementation of the three visualization modes.
  - `src/ui/`: Drawing primitives, widgets, help overlay, and log panel.

## Development Status

The project has completed **Phases 1–6** (Skeleton, Configuration, TLE Pipeline, Propagation, Overhead Map, Sky View). The Sky View (key `2`) is a polar az/el plot with Sun and Moon positions, labeled space stations, elevation rings, and cardinal direction labels. Core Phase 8 items are also done: help overlay, rolling log file, in-app log viewer, config.toml.example, screen-space navigation in both views, and space station highlighting. The `Project Plan.md` provides a detailed roadmap for the remaining phases.

## Building and Running

As a Rust project, the standard commands apply:

- **Build:** `cargo build`
- **Release Build:** `cargo build --release`
- **Run:** `cargo run --release`
- **Test:** `cargo test`
- **Lint:** `cargo clippy` (must pass with `-D warnings`)

## Key Files

- `Project Plan.md`: Comprehensive roadmap, architecture details, and implementation tasks.
- `README.md`: General overview, installation, and usage instructions.
- `config.toml`: User configuration for location and display preferences.
- `config.toml.example`: Example config file with defaults (git-ignored).
- `cache/`: Local TLE data cache.
- `satscanner.log`: Rolling debug log (auto-rotated at 1 MB, git-ignored).

## Development Conventions

- **Safety & Quality:** Use `cargo clippy` and `cargo fmt`. All warnings must be resolved before committing.
- **Testing:** Unit tests for TLE parsing, Sun/Moon ephemeris (sanity checks for day/night, range validation).
- **Async:** Use `tokio` for background tasks like TLE fetching and batch propagation to keep the UI responsive.
- **Logging:** Use `tracing` macros (`info!`, `warn!`, `error!`, `debug!`, `trace!`). Captured in the in-app log viewer (`l`) and rolling file (`satscanner.log`). Set `RUST_LOG=satscanner=trace` for verbose output.
- **Coordination:** Adhere to the phases outlined in `Project Plan.md`.

## Interactive Controls

- `q` / `Ctrl+C` / `Esc`: Exit.
- `1`, `2`, `3`: Switch views (Overhead / Sky / Bands).
- `Arrow Keys`: Screen-space navigation — selects the satellite closest to the pressed direction on screen. Works in both Overhead and Sky views.
- `+` / `-`: Zoom in and out (Overhead view only).
- `?`: Toggle help overlay.
- `l`: Toggle in-app event log viewer.

## Source Modules

| Module | Purpose |
|---|---|
| `src/main.rs` | Entry point, terminal lifecycle, event loop |
| `src/app.rs` | State machine, view routing, tick logic, screen-space navigation |
| `src/config.rs` | TOML config loading |
| `src/log.rs` | In-memory ring buffer + rolling file log (tracing writer) |
| `src/satellite/coords.rs` | ECI → ECEF → Geodetic, Julian date, GMST, observer look angles |
| `src/satellite/propagate.rs` | SGP4 batch propagation |
| `src/satellite/skypos.rs` | Sun and Moon ephemeris (az/el for observer) |
| `src/satellite/tle.rs` | TLE fetch, parse, cache |
| `src/views/overhead.rs` | Equirectangular map view |
| `src/views/globe_scale.rs` | Sky View polar plot |
| `src/views/globe_bands.rs` | Altitude cross-section stub |
| `src/ui/help.rs` | Help overlay |
| `src/ui/log_panel.rs` | In-app log viewer |
| `src/ui/widgets.rs` | Status bar, sidebar |