use crate::views::View;
use crate::config::Config;
use crate::log::SharedLog;
use crate::satellite::tle::{self, Tle};
use crate::satellite::propagate::{self, SatState};
use crate::satellite::coords::Geodetic;
use tokio::sync::oneshot;
use chrono::Utc;

pub struct App {
    pub current_view: View,
    pub is_fetching_tles: bool,
    pub loaded_tles: usize,
    pub config: Config,
    pub log: SharedLog,
    pub tles: Vec<Tle>,
    pub tle_receiver: Option<oneshot::Receiver<Vec<Tle>>>,
    pub sat_states: Vec<SatState>,
    pub selected_overhead_idx: Option<usize>,
    pub zoom_level: f64,
    pub show_help: bool,
    pub show_log: bool,
}

impl App {
    pub fn new(log: SharedLog) -> Self {
        Self {
            current_view: View::Overhead,
            is_fetching_tles: false,
            loaded_tles: 0,
            config: Config::load(),
            log,
            tles: Vec::new(),
            tle_receiver: None,
            sat_states: Vec::new(),
            selected_overhead_idx: None,
            zoom_level: 60.0,
            show_help: false,
            show_log: false,
        }
    }

    pub fn init(&mut self) {
        self.is_fetching_tles = true;
        let (tx, rx) = oneshot::channel();
        self.tle_receiver = Some(rx);

        tracing::info!("Starting TLE fetch from Celestrak...");
        
        tokio::spawn(async move {
            match tle::fetch_tles().await {
                Ok(tles) => {
                    tracing::info!("TLE fetch complete: {} satellites loaded", tles.len());
                    let _ = tx.send(tles);
                }
                Err(e) => {
                    tracing::error!("Error fetching TLEs: {}", e);
                    let _ = tx.send(Vec::new()); // send empty to stop spinner
                }
            }
        });
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_log(&mut self) {
        self.show_log = !self.show_log;
    }

    pub fn set_view(&mut self, view: View) {
        self.current_view = view;
    }

    pub fn zoom_in(&mut self) {
        if self.current_view == View::Overhead {
            self.zoom_level = (self.zoom_level - 10.0).max(10.0);
        }
    }

    pub fn zoom_out(&mut self) {
        if self.current_view == View::Overhead {
            self.zoom_level = (self.zoom_level + 10.0).min(180.0);
        }
    }
    
    pub fn navigate_spatial(&mut self, dx: f64, dy: f64) {
        if self.current_view != View::Overhead { return; }
        
        let overhead_indices: Vec<usize> = self.sat_states.iter().enumerate()
            .filter(|(_, s)| s.el > 0.0)
            .map(|(i, _)| i)
            .collect();
            
        if overhead_indices.is_empty() {
            self.selected_overhead_idx = None;
            return;
        }

        let mut current_lon = self.config.lon;
        let mut current_lat = self.config.lat;

        if let Some(idx) = self.selected_overhead_idx
            && let Some(state) = self.sat_states.get(idx)
        {
            current_lon = state.geodetic.lon;
            current_lat = state.geodetic.lat;
        }

        let mut best_idx = None;
        let mut best_score = f64::INFINITY;

        // Normalize the pressed direction
        let dir_mag = (dx * dx + dy * dy).sqrt();
        let dx_u = dx / dir_mag;
        let dy_u = dy / dir_mag;

        for &idx in &overhead_indices {
            if Some(idx) == self.selected_overhead_idx { continue; }
            
            let state = &self.sat_states[idx];
            let d_lon = state.geodetic.lon - current_lon;
            let d_lat = state.geodetic.lat - current_lat;
            let dist = (d_lon * d_lon + d_lat * d_lat).sqrt();
            if dist < 1e-9 { continue; }

            // Unit vector from current position toward the candidate
            let to_lon = d_lon / dist;
            let to_lat = d_lat / dist;

            // Alignment: dot product of the two unit vectors ∈ [-1, 1]
            let alignment = to_lon * dx_u + to_lat * dy_u;

            // Only consider satellites in the forward hemisphere
            if alignment > 0.0 {
                // Balance alignment with distance:
                //   score = dist × (3 - 2 × alignment)
                // Perfectly aligned (alignment=1): score = dist × 1   (just distance)
                // 60° off       (alignment=0.5): score = dist × 2   (2× penalty)
                // 90° off       (alignment=0):   score = dist × 3   (3× penalty)
                // This keeps nearby satellites strongly preferred while still
                // respecting the pressed direction.
                let penalty = 3.0 - 2.0 * alignment;
                let score = dist * penalty;

                if score < best_score {
                    best_score = score;
                    best_idx = Some(idx);
                }
            }
        }

        if best_idx.is_some() {
            self.selected_overhead_idx = best_idx;
        } else if self.selected_overhead_idx.is_none() {
            self.selected_overhead_idx = Some(overhead_indices[0]);
        }
    }

    pub fn tick(&mut self) {
        if let Some(ref mut rx) = self.tle_receiver {
            match rx.try_recv() {
                Ok(tles) => {
                    tracing::info!("Received {} TLEs via oneshot", tles.len());
                    self.loaded_tles = tles.len();
                    self.tles = tles;
                    self.is_fetching_tles = false;
                    self.tle_receiver = None;
                }
                Err(oneshot::error::TryRecvError::Empty) => { /* not ready yet */ }
                Err(oneshot::error::TryRecvError::Closed) => {
                    tracing::warn!("TLE sender closed without sending data");
                    self.is_fetching_tles = false;
                    self.tle_receiver = None;
                }
            }
        }

        if !self.tles.is_empty() {
            let now = Utc::now();
            let obs = Geodetic {
                lat: self.config.lat,
                lon: self.config.lon,
                alt: self.config.alt,
            };
            self.sat_states = propagate::propagate_all(&self.tles, now, obs);
            tracing::trace!("Propagated {} satellite states", self.sat_states.len());
            
            // Validate selection
            if let Some(idx) = self.selected_overhead_idx
                && idx < self.sat_states.len()
                && self.sat_states[idx].el <= 0.0
            {
                // Deselect if no longer overhead
                self.selected_overhead_idx = None;
            }
        }
    }
}
