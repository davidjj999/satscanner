use crate::satellite::tle::Tle;
use crate::satellite::coords::{self, Geodetic};
use chrono::{DateTime, Utc};
use sgp4::Constants;

#[derive(Debug, Clone)]
pub struct SatState {
    pub name: String,
    #[allow(dead_code)]
    pub eci_pos: [f64; 3],
    pub eci_vel: [f64; 3],
    #[allow(dead_code)]
    pub ecef_pos: [f64; 3],
    pub geodetic: Geodetic,
    pub az: f64,
    pub el: f64,
    pub range: f64,
}

pub fn propagate_all(tles: &[Tle], time: DateTime<Utc>, obs: Geodetic) -> Vec<SatState> {
    let mut states = Vec::with_capacity(tles.len());

    for tle in tles {
        if let Ok(constants) = Constants::from_elements(&tle.elements) {
            let epoch_dt = DateTime::from_naive_utc_and_offset(tle.elements.datetime, Utc);
            let mins_since_epoch = (time - epoch_dt).num_seconds() as f64 / 60.0;

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
