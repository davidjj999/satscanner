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
orbitui/
├── Cargo.toml
├── config.toml              # User location, refresh interval, display prefs
├── src/
│   ├── main.rs              # Entry point, terminal setup, event loop
│   ├── app.rs               # App state machine, view routing, tick logic
│   ├── config.rs            # Config loading and defaults
│   │
│   ├── satellite/
│   │   ├── mod.rs
│   │   ├── tle.rs           # TLE fetch, parse, cache (Celestrak HTTP)
│   │   ├── propagate.rs     # SGP4 wrapper, batch position compute
│   │   └── coords.rs        # ECI → ECEF → Geodetic conversions
│   │
│   ├── views/
│   │   ├── mod.rs           # View enum, shared render helpers
│   │   ├── overhead.rs      # View 1: 2D equirectangular map
│   │   ├── globe_scale.rs   # View 2: Orthographic globe, true altitude
│   │   └── globe_bands.rs   # View 3: Orthographic globe, log-scaled altitude
│   │
│   └── ui/
│       ├── mod.rs
│       ├── canvas.rs        # Braille canvas drawing primitives
│       ├── projection.rs    # Orthographic, equirectangular projection math
│       ├── colors.rs        # Satellite type → color mapping
│       └── widgets.rs       # Info sidebar, status bar, keybind overlay
```

---

## Phases

---

### Phase 1 — Project Skeleton & Terminal Setup [COMPLETED]
**Estimated time: 2–3 days**

Stand up the Rust project with all dependencies declared, get a blank TUI rendering with a proper event loop, and confirm clean teardown on exit.

**Tasks:**
- `cargo new orbitui` and populate `Cargo.toml` with all required dependencies
- Implement terminal initialization: raw mode, alternate screen, panic hook that restores terminal
- Main loop: render tick (configurable FPS) + input event polling using `crossterm`
- Implement graceful exit on `q` / `Ctrl+C`
- Stub out three view modules returning placeholder frames
- Implement view switching on `1`, `2`, `3` keypresses
- Render a basic status bar (current view name, time, placeholder satellite count)
- Handle terminal resize events (redraw on `SIGWINCH` / crossterm resize event)

**Crates:**
```toml
ratatui = "0.27"
crossterm = "0.27"
tokio = { version = "1", features = ["full"] }
```

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

**Crates:**
```toml
serde = { version = "1", features = ["derive"] }
toml = "0.8"
```

---

### Phase 3 — TLE Data Pipeline [COMPLETED]
**Estimated time: 2–3 days**

Fetch Two-Line Element sets from Celestrak, parse them, and cache them locally so the app works offline after first run.

**Tasks:**
- Define `Tle` struct mirroring SGP4 input requirements
- HTTP fetch from Celestrak endpoints (active satellites, Starlink, GPS, ISS as initial sets)
  - `https://celestrak.org/SOCRATES/query.php` for active satellites
  - `https://celestrak.org/SOCRATES/` group URLs for curated sets
- Parse raw 3-line TLE text format into `Tle` structs
- Write parsed TLEs to a local cache file (`~/.cache/orbitui/tle_<group>.txt`)
- Cache invalidation: re-fetch if cache is older than 2 hours
- Async fetch using `reqwest` + `tokio`; show "Fetching TLE data..." spinner on first run
- Error handling: if fetch fails and cache exists, use stale cache with warning; if no cache, exit with message

**Crates:**
```toml
reqwest = { version = "0.12", features = ["rustls-tls"] }
chrono = { version = "0.4", features = ["serde"] }
dirs = "5"
```

**TLE sources:**
| Group | URL |
|---|---|
| Active satellites | `https://celestrak.org/SOCRATES/` |
| ISS | `https://celestrak.org/satcat/tle.php?CATNR=25544` |
| GPS | `https://celestrak.org/gnss/gps/` |
| Starlink | `https://celestrak.org/satcat/tle.php?GROUP=starlink` |
| Weather | `https://celestrak.org/weather/` |

---

### Phase 4 — Orbital Propagation & Coordinate Math [COMPLETED]
**Estimated time: 3–4 days**

The core computational engine. Given a TLE and a timestamp, produce a lat/lon/altitude for each satellite.

**Tasks:**

**4a — SGP4 Propagation:** [COMPLETED]
- Integrate the `sgp4` crate
- Implement `propagate_all(tles: &[Tle], time: DateTime<Utc>) -> Vec<SatState>` where `SatState` holds ECI position + velocity
- Batch over all loaded TLEs each tick (target <16ms for up to 2000 satellites)
- Handle propagation errors gracefully (skip satellites with bad TLEs)

**4b — Coordinate Conversions:** [COMPLETED]
- ECI (km) → ECEF (km): requires Greenwich Sidereal Time (GST) rotation matrix
  - GST formula: Julian Date → GMST → add Earth rotation since J2000
- ECEF (km) → Geodetic (lat, lon, alt):
  - Use iterative Bowring method or Zhu's closed-form for WGS84
- Expose `eci_to_geodetic(pos: Vec3, time: DateTime<Utc>) -> Geodetic`

**4c — Observer Geometry:** [COMPLETED]
- Compute azimuth/elevation from observer location to each satellite
- Flag satellites with elevation > 0° as "overhead" (visible from user's location)
- Compute range (km) and Doppler-corrected velocity for sidebar display

**Crates:**
```toml
sgp4 = "2"
```

**Exit criteria:** ISS position printed to terminal matches Heavens-Above within ~10 km.

---

### Phase 5 — View 1: Overhead Map [COMPLETED]
**Estimated time: 2–3 days**

A 2D equirectangular world map showing all satellites as dots, with the observer's location marked and overhead satellites highlighted.

**Tasks:**
- Use ratatui's built-in `Map` widget as the base layer (renders world coastlines) [COMPLETED]
- Project each satellite's (lat, lon) onto canvas (x, y) using equirectangular mapping [COMPLETED]
- Render all satellites as dim dots; overhead satellites (el > 0°) as bright colored dots [COMPLETED]
- Mark observer location with a distinct symbol (e.g., `⊕`) [COMPLETED]
- Zoom map to observer's location with aspect ratio correction [COMPLETED]
- Spatial navigation: arrow keys move selection to the nearest satellite in that direction [COMPLETED]
- Sidebar panel: selected satellite details (name, NORAD ID, alt km, az/el, range, velocity) [COMPLETED]
- Color-code by satellite type: LEO=cyan, MEO=yellow, GEO=magenta, HEO=red [COMPLETED]
- Support `+` / `-` keys to filter by satellite group [PENDING]
- Show satellite count in status bar [COMPLETED]

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

**Coastline data:** Embed Natural Earth 110m coastline as a `&[(f32, f32)]` array generated at build time via a build script from the GeoJSON source. Compressed it is ~50 KB.

---

### Phase 7 — View 3: Altitude-Banded Globe
**Estimated time: 2–3 days**

Same orthographic globe but with altitude scaled logarithmically so LEO, MEO, and GEO satellites are all visible with clear spatial separation.

**Tasks:**
- Reuse View 2 projection infrastructure with a pluggable radial scale function
- Implement log-scale: `r_screen = R_earth + scale_factor * log2(1 + alt_km / 1000.0)`
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

**Performance:**
- Profile propagation loop; ensure <16ms for 2000 satellites on a modern machine
- Move TLE fetch and propagation to background tokio tasks; send updates to render thread via `tokio::sync::watch`
- Cache projected positions per frame; only recompute when time advances or view rotates
- Coastline projection cache keyed on camera angle (invalidate on rotation)

**Polish:**
- Startup screen: show progress bar while fetching TLEs
- Help overlay: `?` key shows keybind reference
- Error toast: non-fatal errors (stale TLE, fetch failure) shown in status bar for 3 seconds
- Config reload: `R` reloads config and re-fetches TLEs
- Graceful degradation: if terminal is too small (<80×24), show a "Terminal too small" message
- Mouse support (optional): click to select satellite

**Testing:**
- Unit tests for coordinate conversion (compare against known ephemeris values)
- Unit tests for TLE parsing (use sample TLEs from Celestrak docs)
- Integration test: propagate ISS TLE, assert position within 50 km of reference

---

### Phase 9 — Packaging & Documentation
**Estimated time: 1–2 days**

- `README.md`: installation, config file format, keybind reference, screenshot/demo GIF
- `CHANGELOG.md`
- `config.toml.example`
- GitHub Actions CI: `cargo build`, `cargo test`, `cargo clippy` on Linux/macOS/Windows
- Cargo release profile: `opt-level = 3`, `lto = true`, `codegen-units = 1`
- Publish to crates.io (optional)

---

## Full Dependency List

```toml
[dependencies]
# TUI
ratatui = "0.27"
crossterm = "0.27"

# Async runtime
tokio = { version = "1", features = ["full"] }

# HTTP
reqwest = { version = "0.12", features = ["rustls-tls", "json"] }

# Orbital mechanics
sgp4 = "2"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Config / serialization
serde = { version = "1", features = ["derive"] }
toml = "0.8"

# Filesystem / paths
dirs = "5"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
approx = "0.5"   # floating point assertions in coord tests
```

---

## Key Algorithms Reference

### Equirectangular Projection (View 1)
```
x = (lon + 180) / 360 * canvas_width
y = (90 - lat) / 180 * canvas_height
```

### Orthographic Projection (Views 2 & 3)
```
# Convert geodetic to ECEF unit sphere
X = cos(lat) * cos(lon)
Y = cos(lat) * sin(lon)
Z = sin(lat)

# Scale by radial distance (true or log-scaled)
r = (R_earth + alt_km) / R_earth   # normalized, or log-scaled for View 3
P = r * (X, Y, Z)

# Rotate by camera (azimuth φ, elevation θ)
P' = Ry(θ) * Rz(φ) * P

# Project to screen (orthographic: just drop Z)
screen_x = P'.x * scale + cx
screen_y = -P'.y * scale + cy   # y-flip for screen coords

# Occlusion: skip if P'.z < 0
```

### ECI → Geodetic Conversion
```
# 1. Compute Greenwich Mean Sidereal Time
JD = julian_date(utc_time)
T = (JD - 2451545.0) / 36525.0
GMST_deg = 280.46061837 + 360.98564736629 * (JD - 2451545.0)
             + 0.000387933 * T^2 - T^3 / 38710000.0

# 2. ECI → ECEF (rotate by -GMST around Z axis)
x_ecef =  x_eci * cos(GMST) + y_eci * sin(GMST)
y_ecef = -x_eci * sin(GMST) + y_eci * cos(GMST)
z_ecef =  z_eci

# 3. ECEF → Geodetic (Zhu closed-form for WGS84)
# See: Zhu, J. (1994). Conversion of Earth-centered Earth-fixed coordinates
#      to geodetic coordinates. IEEE Transactions on Aerospace and Electronic Systems.
```

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

**Total estimated time: 18–26 days** of focused development, assuming familiarity with Rust but not with orbital mechanics or TUI frameworks.

---

## Known Risks & Mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| SGP4 coordinate errors are subtle and hard to spot | Medium | Validate against Heavens-Above for ISS early in Phase 4 |
| Braille canvas resolution too low for globe detail | Low | Test on 220×50 canvas early; coastline LOD is adjustable |
| Celestrak rate-limits aggressive fetchers | Low | Cache aggressively; add `User-Agent` header; respect 2hr TTL |
| Terminal rendering differs across platforms | Medium | Test on macOS Terminal, iTerm2, Windows Terminal, GNOME Terminal |
| Propagating 5000+ Starlink sats causes lag | Medium | Profile early; consider spatial culling (only propagate within view frustum) |