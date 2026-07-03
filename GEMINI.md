# Satellite Tracker TUI (satscanner) — Context

This project, `satscanner`, is a terminal-based satellite tracking application written in Rust. It uses SGP4 orbital propagation and live TLE data from Celestrak to provide real-time satellite positions across three interactive views: a 2D overhead map, a to-scale 3D globe, and an altitude-banded globe.

## Project Overview

- **Core Technologies:** Rust, `ratatui` (TUI), `crossterm` (Terminal handling), `tokio` (Async runtime), `sgp4` (Orbital mechanics), `reqwest` (HTTP), `serde`/`toml` (Config).
- **Architecture:** 
  - `src/main.rs`: Entry point, terminal setup, and event loop.
  - `src/app.rs`: State machine, view routing, and tick logic.
  - `src/log.rs`: In-memory ring buffer + rolling file log, wired to `tracing`.
  - `src/config.rs`: Config loading from `config.toml` with defaults.
  - `src/satellite/`: TLE fetching, parsing, and SGP4 propagation.
  - `src/views/`: Implementation of the three visualization modes.
  - `src/ui/`: Drawing primitives, projections, widgets, help overlay, and log panel.

## Development Status

The project has completed **Phases 1–5** (Skeleton, Configuration, TLE Pipeline, Propagation, and Overhead Map). Core Phase 8 items are also done: **help overlay** (`?`), **rolling log file** (`satscanner.log`), **in-app log viewer** (`l`), and **`config.toml.example`**. Real-time satellite tracking with spatial navigation is functional in the Overhead view. The `Project Plan.md` provides a detailed roadmap for the remaining phases.

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
- `config.toml.example`: Example config file with defaults (git-ignored from `.gitignore`).
- `cache/`: Local TLE data cache.
- `satscanner.log`: Rolling debug log (auto-rotated at 1 MB, git-ignored).

## Development Conventions

- **Safety & Quality:** Use `cargo clippy` and `cargo fmt`. All warnings must be resolved before committing.
- **Testing:** Unit tests are required for coordinate conversions and TLE parsing.
- **Async:** Use `tokio` for background tasks like TLE fetching and batch propagation to keep the UI responsive.
- **Logging:** Use `tracing` macros (`info!`, `warn!`, `error!`, `debug!`, `trace!`). Output is captured in the in-app log viewer (`l`) and the rolling file (`satscanner.log`). Set `RUST_LOG=satscanner=trace` for verbose output.
- **Coordination:** Adhere to the phases outlined in `Project Plan.md`.

## Interactive Controls

- `q` / `Ctrl+C` / `Esc`: Exit.
- `1`, `2`, `3`: Switch views (Overhead / Globe Scale / Globe Bands).
- `Arrow Keys`: Spatial navigation — select the satellite closest to the pressed direction from the current selection (Overhead view).
- `+` / `-`: Zoom in and out (Overhead view).
- `?`: Toggle help overlay.
- `l`: Toggle in-app event log viewer.