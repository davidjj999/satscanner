# Satellite Tracker TUI — Project Plan

## Overview

A terminal-based satellite tracking application written in Rust, featuring three interactive views: a 2D overhead map, a to-scale 3D globe, and an altitude-banded globe. Satellite positions are computed in real time using SGP4 orbital propagation against live TLE data fetched from Celestrak.

---

## Goals & Non-Goals

### Goals
- Real-time satellite position tracking rendered entirely in the terminal
- Three switchable views covering different use cases (situational awareness, orbital mechanics, altitude regimes)
- Smooth keyboard-driven navigation and interaction
- Zero cost to run (free data sources, no API keys required for basic use)
- Cross-platform: Linux, macOS, Windows (via crossterm)

### Non-Goals
- GUI or web frontend
- Proprietary data sources or paid APIs
- Satellite signal decoding or RF features
- Launch prediction or maneuver planning
- Mobile support

---

## Architecture

```
satscanner/
├── Cargo.toml
├── config.toml              # User location (git-ignored)
├── config.toml.example      # Example config file
├── satscanner.log           # Rolling debug log (git-ignored)
├── cache/
│   └── tle_active.txt       # Cached TLE data
├── src/
│   ├── main.rs              # Entry point, terminal setup, event loop
│   ├── app.rs               # App state machine, view routing, tick logic
│   ├── config.rs            # Config loading and defaults
│   ├── log.rs               # In-memory ring buffer + rolling file log (tracing)
│   │
│   ├── satellite/
│   │   ├── mod.rs
│   │   ├── tle.rs           # TLE fetch, parse, cache (Celestrak HTTP)
│   │   ├── propagate.rs     # SGP4 wrapper, batch position compute
│   │   └── coords.rs        # ECI → ECEF → Geodetic conversions
│   │
│   ├── views/
│   │   ├── mod.rs           # View enum
│   │   ├── overhead.rs      # View 1: 2D equirectangular map
│   │   ├── globe_scale.rs   # View 2: Orthographic globe, true altitude (stub)
│   │   └── globe_bands.rs   # View 3: Orthographic globe, log-scaled altitude (stub)
│   │
│   └── ui/
│       ├── mod.rs           # Top-level draw routing
│       ├── widgets.rs       # Status bar, info sidebar
│       ├── help.rs          # Help overlay popup (? key)
│       └── log_panel.rs     # In-app log viewer overlay (l key)
```

---

## Phases

### Phase 1 — Project Skeleton & Terminal Setup [COMPLETED]
**Estimated time: 2–3 days**

Stand up the Rust project with all dependencies declared, get a blank TUI rendering with a proper event loop, and confirm clean teardown on exit.

**Tasks:**
- `cargo new satscanner` and populate `Cargo.toml` with all required dependencies
- Implement terminal initialization: raw mode, alternate screen, panic hook that restores terminal
- Main loop: render tick (configurable FPS) + input event polling using `crossterm`
- Implement graceful exit on `q` / `Ctrl+C`
- Stub out three view modules returning placeholder frames
- Implement view switching on `1`, `2`, `3` keypresses
- Render a basic status bar (current view name, time, placeholder satellite count)
- Handle terminal resize events (redraw on `SIGWINCH` / crossterm resize event)

**Exit criteria:** App launches, shows placeholder views, switches between them, exits cleanly.

---

### Phase 2 — Configuration & Location [COMPLETED]
**Estimated time: 1 day**

Allow the user to specify their observer location, which drives the overhead view and pass predictions.

**Tasks:**
- Define `Config` struct: latitude, longitude, altitude (meters), location name
- Load from `config.toml` using `toml` + `serde`
- Fall back to hardcoded defaults (0.0, 0.0) if no config found
- Expose config to all views via app state
- Document config file format in `README.md`
- Provide `config.toml.example` with commented defaults

---

### Phase 3 — TLE Data Pipeline [COMPLETED]
**Estimated time: 2–3 days**

Fetch Two-Line Element sets from Celestrak, parse them, and cache them locally so the app works offline after first run.

**Tasks:**
- Define `Tle` struct mirroring SGP4 input requirements
- HTTP fetch from Celestrak (active satellite group)
- Parse raw 3-line TLE text format into `Tle` structs using `sgp4::Elements::from_tle`
- Write parsed TLEs to a local cache file (`cache/tle_active.txt`)
- Cache invalidation: re-fetch if cache is older than 2 hours
- Async fetch using `reqwest` + `tokio`; show "Fetching TLE data..." in status bar
- Error handling: if fetch fails and cache exists, use stale cache with warning; if no cache, exit with error
- **Data delivery:** TLEs sent from async task to main loop via `tokio::sync::oneshot`

---

### Phase 4 — Orbital Propagation & Coordinate Math [COMPLETED]
**Estimated time: 3–4 days**

The core computational engine. Given a TLE and a timestamp, produce a lat/lon/altitude for each satellite.

**Tasks:**

**4a — SGP4 Propagation:** [COMPLETED]
- Integrate the `sgp4` crate
- Implement `propagate_all(tles: &[Tle], time: DateTime<Utc>) -> Vec<SatState>`
- Batch over all loaded TLEs each tick (target <16ms for up to 2000 satellites)
- Handle propagation errors gracefully (skip satellites with bad TLEs)
- Epoch read from `Elements.datetime` (not manually parsed from TLE text)

**4b — Coordinate Conversions:** [COMPLETED]
- ECI (km) → ECEF (km): Greenwich Sidereal Time rotation matrix
  - GST formula: Julian Date → GMST → add Earth rotation since J2000
- ECEF (km) → Geodetic (lat, lon, alt): Zhu's closed-form solution for WGS84
- Expose `ecef_to_geodetic(pos: [f64;3]) -> Geodetic`

**4c — Observer Geometry:** [COMPLETED]
- Compute azimuth/elevation from observer location to each satellite
- Flag satellites with elevation > 0° as "overhead" (visible from user's location)
- Compute range (km) and velocity for sidebar display

**Exit criteria:** ISS position printed to terminal matches Heavens-Above within ~10 km.

---

### Phase 5 — View 1: Overhead Map [COMPLETED]
**Estimated time: 2–3 days**

A 2D equirectangular world map showing all satellites as dots, with the observer's location marked and overhead satellites highlighted.

**Tasks:**
- Use ratatui's built-in `Map` widget as the base layer (renders world coastlines) [COMPLETED]
- Project each satellite's (lat, lon) onto canvas (x, y) using equirectangular mapping [COMPLETED]
- Render all satellites as dim dots; overhead satellites (el > 0°) as bright colored dots [COMPLETED]
- Mark observer location with a distinct symbol (`⊕`) [COMPLETED]
- Zoom map to observer's location with aspect ratio correction and equatorial cosine compensation [COMPLETED]
- Spatial navigation: arrow keys move selection to nearest satellite in that direction [COMPLETED]
  - Uses unit-vector alignment scoring: `score = dist × (3 − 2 × alignment)`
  - Prefers nearby satellites, penalizes off-axis direction
- Sidebar panel: selected satellite details (name, alt km, az/el, range, velocity) [COMPLETED]
- Color-code by satellite type: LEO=cyan, MEO=yellow, GEO=magenta, HEO=red [COMPLETED]
- Support `+` / `-` keys to zoom in/out [COMPLETED]
- Show satellite count in status bar [COMPLETED]
- Selected satellite excluded from Points layers to avoid sub-character rendering drift [COMPLETED]

**Rendering approach:**
```
Canvas widget → equirectangular projection → Points primitives
Map widget for coastlines (layered under satellite points)
```

---

### Phase 6 — View 2: To-Scale Globe
**Estimated time: 3–4 days**

An orthographic globe projection showing Earth and satellite orbits at true scale. A GEO satellite appears roughly 6× the Earth's radius from center.

**Tasks:**
- Implement orthographic projection: `(lat, lon, alt) → (screen_x, screen_y)` with configurable camera azimuth/elevation
- Draw Earth outline as a circle (radius proportional to terminal canvas size)
- Render continental outlines by projecting coastline vertex list through orthographic projection (use a bundled low-resolution coastline dataset, e.g., Natural Earth 110m as embedded binary)
- Render each satellite as a point at its true radial distance from Earth center
- Draw orbit traces: compute 90-minute track (one orbital period) as a polyline of projected points
- Occlude satellites behind Earth (dot product with view vector < 0 → skip)
- Keyboard controls:
  - `←` `→` — rotate globe longitude
  - `↑` `↓` — rotate globe latitude (tilt)
  - `r` — reset to default view (observer-centered)
  - `f` — toggle orbit trace on/off
- Animate: globe rotates 0.25°/tick in play mode; `Space` toggles play/pause
- Sidebar: same satellite detail panel as View 1

**Coastline data:** Embed Natural Earth 110m coastline as a `&[(f32, f32)]` array generated at build time via a build script from the GeoJSON source (~50 KB compressed).

---

### Phase 7 — View 3: Altitude-Banded Globe
**Estimated time: 2–3 days**

Same orthographic globe but with altitude scaled logarithmically so LEO, MEO, and GEO satellites are all visible with clear spatial separation.

**Tasks:**
- Reuse View 2 projection infrastructure with a pluggable radial scale function
- Implement log-scale: `r_screen = R_earth + scale_factor × log2(1 + alt_km / 1000.0)`
- Draw labeled altitude band rings:
  - LEO band: 160–2000 km
  - MEO band: 2000–35,786 km  
  - GEO ring: 35,786 km (geosynchronous)
  - HEO: indicate apogee/perigee arc for elliptical orbits
- Render band labels on the right edge of the globe: "LEO", "MEO", "GEO"
- Shade bands with subtle background fill (using braille density patterns)
- All View 2 rotation controls apply here
- Add `[` `]` keys to adjust the log scale exponent interactively
- Color dots by altitude regime rather than satellite type in this view

---

### Phase 8 — Polish & Performance
**Estimated time: 2–3 days**

**Tasks completed:**
- [x] Help overlay (`?` key shows keybind reference and color legend)
- [x] `config.toml.example` shipped with the repo
- [x] Rolling file log (`satscanner.log`, 1 MB auto-rotation, one backup kept)
- [x] In-app log viewer (`l` key), captures all `tracing` output
- [x] TLE delivery uses `tokio::sync::oneshot` (correct single-message semantics)
- [x] Fixed spatial navigation scoring to balance alignment with distance
- [x] Selected satellite excluded from Points layers to fix sub-character rendering drift
- [x] `cargo clippy` passes cleanly with `-D warnings`
- [x] All unused-code warnings resolved with `#[allow(dead_code)]` where intentional
- [x] Collapsible-if lints resolved with Rust 2024 let-chains

**Tasks remaining:**
- [ ] Profile propagation loop; ensure <16ms for 2000 satellites
- [ ] Cache projected positions per frame
- [ ] Startup screen: progress bar while fetching TLEs
- [ ] Error toast in status bar for 3 seconds
- [ ] Config reload (`R` key)
- [ ] Graceful degradation if terminal too small (<80×24)
- [ ] Mouse support (click to select satellite)
- [ ] Unit tests for coordinate conversion
- [ ] More TLE parsing test coverage
- [ ] Integration test: propagate ISS TLE within 50 km of reference

---

### Phase 9 — Packaging & Documentation
**Estimated time: 1–2 days**

- `README.md`: installation, config file format, keybind reference, screenshot/demo GIF [DONE]
- `CHANGELOG.md`
- GitHub Actions CI: `cargo build`, `cargo test`, `cargo clippy` on Linux/macOS/Windows
- Cargo release profile: `opt-level = 3`, `lto = true`, `codegen-units = 1`
- Publish to crates.io (optional)

---

## Full Dependency List

```toml
[dependencies]
ratatui = "0.27"
crossterm = "0.27"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["rustls-tls", "json"] }
sgp4 = "2"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

---

## Key Algorithms Reference

### Equirectangular Projection (View 1)
```
x = (lon + 180) / 360 × canvas_width
y = (90 − lat) / 180 × canvas_height
```

### Orthographic Projection (Views 2 & 3)
```
X = cos(lat) × cos(lon)
Y = cos(lat) × sin(lon)
Z = sin(lat)

r = (R_earth + alt_km) / R_earth    # normalized, or log-scaled for View 3
P = r × (X, Y, Z)

P' = Ry(θ) × Rz(φ) × P              # camera rotation

screen_x = P'.x × scale + cx
screen_y = −P'.y × scale + cy        # y-flip
```

### Spatial Navigation Scoring (View 1)
```
alignment = to_lon × dx_u + to_lat × dy_u   # unit-vector dot product
score     = dist × (3.0 − 2.0 × alignment)  # lower = better
```
Perfectly aligned (alignment = 1):  `score = dist × 1.0`
60° off (alignment = 0.5):          `score = dist × 2.0`
90° off (alignment = 0):            `score = dist × 3.0`

---

## Coordinate System Summary

| Frame | Origin | Used for |
|---|---|---|
| ECI (J2000) | Earth center, fixed to stars | SGP4 output |
| ECEF | Earth center, fixed to Earth | Intermediate |
| Geodetic (WGS84) | Earth surface | Lat/lon/alt display |
| Observer (az/el) | Observer on surface | Overhead visibility |

---

## Milestone Summary

| Milestone | Phases | Deliverable |
|---|---|---|
| M1 — Skeleton | 1–2 | [DONE] App boots, switches views, reads config |
| M2 — Data | 3–4 | [DONE] Live satellite data fetched and propagated via SGP4 |
| M3 — View 1 | 5 | [DONE] Overhead map working end-to-end with spatial navigation |
| M4 — Views 2 & 3 | 6–7 | [IN PROGRESS] Globe views with rotation controls |
| M5 — Ship | 8–9 | Polished, documented, CI passing |

**Total estimated time:** 18–26 days of focused development.

---

## Known Risks & Mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| SGP4 coordinate errors are subtle and hard to spot | Medium | Validate against Heavens-Above for ISS early in Phase 4 |
| Braille canvas resolution too low for globe detail | Low | Test on 220×50 canvas early; coastline LOD is adjustable |
| Celestrak rate-limits aggressive fetchers | Low | Cache aggressively; respect 2hr TTL |
| Terminal rendering differs across platforms | Medium | Test on macOS Terminal, iTerm2, Windows Terminal, GNOME Terminal |
| Propagating 5000+ Starlink sats causes lag | Medium | Profile early; consider spatial culling (only propagate within view frustum) |