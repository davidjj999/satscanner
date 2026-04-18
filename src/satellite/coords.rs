use chrono::{DateTime, Datelike, Timelike, Utc};

const R_EARTH: f64 = 6378.137; // WGS84 Equatorial radius in km
const F: f64 = 1.0 / 298.257223563; // WGS84 flattening
const E2: f64 = F * (2.0 - F); // Square of eccentricity

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geodetic {
    pub lat: f64, // degrees
    pub lon: f64, // degrees
    pub alt: f64, // km
}

pub fn julian_date(time: &DateTime<Utc>) -> f64 {
    let mut y = time.year() as f64;
    let mut m = time.month() as f64;
    let d = time.day() as f64;
    let h = time.hour() as f64;
    let min = time.minute() as f64;
    let sec = time.second() as f64 + time.nanosecond() as f64 / 1e9;

    if m <= 2.0 {
        y -= 1.0;
        m += 12.0;
    }
    
    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();

    let jd = (365.25 * (y + 4716.0)).floor()
        + (30.6001 * (m + 1.0)).floor()
        + d + b - 1524.5;
    
    jd + (h + min / 60.0 + sec / 3600.0) / 24.0
}

pub fn gmst(jd: f64) -> f64 {
    let t = (jd - 2451545.0) / 36525.0;
    let mut gmst = 280.46061837 + 360.98564736629 * (jd - 2451545.0)
        + 0.000387933 * t * t - (t * t * t) / 38710000.0;
    
    // Normalize to 0-360
    gmst %= 360.0;
    if gmst < 0.0 {
        gmst += 360.0;
    }
    gmst.to_radians()
}

pub fn eci_to_ecef(eci: [f64; 3], time: &DateTime<Utc>) -> [f64; 3] {
    let jd = julian_date(time);
    let theta = gmst(jd);
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();

    [
        eci[0] * cos_theta + eci[1] * sin_theta,
        -eci[0] * sin_theta + eci[1] * cos_theta,
        eci[2],
    ]
}

pub fn ecef_to_geodetic(ecef: [f64; 3]) -> Geodetic {
    // Zhu's closed-form solution
    let x = ecef[0];
    let y = ecef[1];
    let z = ecef[2];

    let a = R_EARTH;
    let b = a * (1.0 - F);

    let ep2 = (a * a - b * b) / (b * b);
    let p = (x * x + y * y).sqrt();
    let theta = (z * a).atan2(p * b);
    let st = theta.sin();
    let ct = theta.cos();

    let lon = y.atan2(x);
    let lat = (z + ep2 * b * st * st * st).atan2(p - E2 * a * ct * ct * ct);
    
    let sin_lat = lat.sin();
    let n = a / (1.0 - E2 * sin_lat * sin_lat).sqrt();
    let alt = p / lat.cos() - n;

    Geodetic {
        lat: lat.to_degrees(),
        lon: lon.to_degrees(),
        alt,
    }
}

pub fn eci_to_geodetic(eci: [f64; 3], time: &DateTime<Utc>) -> Geodetic {
    let ecef = eci_to_ecef(eci, time);
    ecef_to_geodetic(ecef)
}

pub fn observer_look_angle(obs: Geodetic, sat_ecef: [f64; 3]) -> (f64, f64, f64) {
    let lat = obs.lat.to_radians();
    let lon = obs.lon.to_radians();

    let sin_lat = lat.sin();
    let cos_lat = lat.cos();
    let sin_lon = lon.sin();
    let cos_lon = lon.cos();

    let a = R_EARTH;
    let n = a / (1.0 - E2 * sin_lat * sin_lat).sqrt();
    
    let obs_x = (n + obs.alt) * cos_lat * cos_lon;
    let obs_y = (n + obs.alt) * cos_lat * sin_lon;
    let obs_z = (n * (1.0 - E2) + obs.alt) * sin_lat;

    let rx = sat_ecef[0] - obs_x;
    let ry = sat_ecef[1] - obs_y;
    let rz = sat_ecef[2] - obs_z;

    let top_s = sin_lat * cos_lon * rx + sin_lat * sin_lon * ry - cos_lat * rz;
    let top_e = -sin_lon * rx + cos_lon * ry;
    let top_z = cos_lat * cos_lon * rx + cos_lat * sin_lon * ry + sin_lat * rz;

    let range = (top_s * top_s + top_e * top_e + top_z * top_z).sqrt();
    let az = top_e.atan2(-top_s).to_degrees();
    let az = if az < 0.0 { az + 360.0 } else { az };
    let el = (top_z / range).asin().to_degrees();

    (az, el, range)
}
