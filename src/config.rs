use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
    pub location_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lat: 0.0,
            lon: 0.0,
            alt: 0.0,
            location_name: "Null Island".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string("config.toml") {
            toml::from_str(&content).unwrap_or_else(|_| Self::default())
        } else {
            Self::default()
        }
    }
}
