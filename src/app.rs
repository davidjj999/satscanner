use crate::views::View;
use crate::config::Config;
use crate::satellite::tle::{self, Tle};
use crate::satellite::propagate::{self, SatState};
use crate::satellite::coords::Geodetic;
use tokio::sync::mpsc;
use chrono::Utc;

pub struct App {
    pub current_view: View,
    pub is_fetching_tles: bool,
    pub loaded_tles: usize,
    pub config: Config,
    pub tles: Vec<Tle>,
    pub tle_receiver: Option<mpsc::UnboundedReceiver<Vec<Tle>>>,
    pub sat_states: Vec<SatState>,
    pub selected_overhead_idx: Option<usize>,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_view: View::Overhead,
            is_fetching_tles: false,
            loaded_tles: 0,
            config: Config::load(),
            tles: Vec::new(),
            tle_receiver: None,
            sat_states: Vec::new(),
            selected_overhead_idx: None,
        }
    }

    pub fn init(&mut self) {
        self.is_fetching_tles = true;
        let (tx, rx) = mpsc::unbounded_channel();
        self.tle_receiver = Some(rx);
        
        tokio::spawn(async move {
            match tle::fetch_tles().await {
                Ok(tles) => {
                    let _ = tx.send(tles);
                }
                Err(e) => {
                    tracing::error!("Error fetching TLEs: {}", e);
                    let _ = tx.send(Vec::new()); // send empty to stop spinner
                }
            }
        });
    }

    pub fn set_view(&mut self, view: View) {
        self.current_view = view;
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

        if let Some(idx) = self.selected_overhead_idx {
            if let Some(state) = self.sat_states.get(idx) {
                current_lon = state.geodetic.lon;
                current_lat = state.geodetic.lat;
            }
        }

        let mut best_idx = None;
        let mut best_score = f64::INFINITY;

        for &idx in &overhead_indices {
            if Some(idx) == self.selected_overhead_idx { continue; }
            
            let state = &self.sat_states[idx];
            let d_lon = state.geodetic.lon - current_lon;
            let d_lat = state.geodetic.lat - current_lat;
            
            let dot = d_lon * dx + d_lat * dy;
            if dot > 0.0 {
                let dist = (d_lon * d_lon + d_lat * d_lat).sqrt();
                let score = dist * dist / dot; 
                
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
            if let Ok(tles) = rx.try_recv() {
                self.loaded_tles = tles.len();
                self.tles = tles;
                self.is_fetching_tles = false;
                self.tle_receiver = None;
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
            
            // Validate selection
            if let Some(idx) = self.selected_overhead_idx {
                if idx < self.sat_states.len() && self.sat_states[idx].el <= 0.0 {
                    // Deselect if no longer overhead
                    self.selected_overhead_idx = None;
                }
            }
        }
    }
}
