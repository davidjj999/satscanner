use crate::satellite::tle::Tle;
use crate::satellite::coords::{self, Geodetic};
use chrono::{DateTime, TimeZone, Utc};
use sgp4::Constants;

#[derive(Debug, Clone)]
pub struct SatState {
    pub name: String,
    pub eci_pos: [f64; 3],
    pub eci_vel: [f64; 3],
    pub ecef_pos: [f64; 3],
    pub geodetic: Geodetic,
    pub az: f64,
    pub el: f64,
    pub range: f64,
}

fn extract_epoch_jd(line1: &str) -> f64 {
    if line1.len() < 32 { return 0.0; }
    let year_str = &line1[18..20];
    let day_str = &line1[20..32];
    let mut year = year_str.trim().parse::<i32>().unwrap_or(0);
    let day = day_str.trim().parse::<f64>().unwrap_or(0.0);
    
    if year < 57 { year += 2000; } else { year += 1900; }
    
    let jan1 = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap();
    let jan1_jd = coords::julian_date(&jan1);
    
    jan1_jd + day - 1.0
}

pub fn propagate_all(tles: &[Tle], time: DateTime<Utc>, obs: Geodetic) -> Vec<SatState> {
    let mut states = Vec::with_capacity(tles.len());
    let current_jd = coords::julian_date(&time);

    for tle in tles {
        if let Ok(constants) = Constants::from_elements(&tle.elements) {
            let epoch_jd = extract_epoch_jd(&tle.line1);
            let mins_since_epoch = (current_jd - epoch_jd) * 1440.0;

            if let Ok(prediction) = constants.propagate(sgp4::MinutesSinceEpoch(mins_since_epoch)) {
                let eci_pos = prediction.position; // assuming it returns [f64; 3]
                let eci_vel = prediction.velocity;
                
                let ecef_pos = coords::eci_to_ecef(eci_pos, &time);
                let geodetic = coords::ecef_to_geodetic(ecef_pos);
                let (az, el, range) = coords::observer_look_angle(obs, ecef_pos);

                states.push(SatState {
                    name: tle.name.clone(),
                    eci_pos,
                    eci_vel,
                    ecef_pos,
                    geodetic,
                    az,
                    el,
                    range,
                });
            }
        }
    }
    
    states
}
