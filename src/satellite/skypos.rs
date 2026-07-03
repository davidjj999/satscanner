use crate::satellite::coords::{self, Geodetic};
use chrono::{DateTime, Utc};

// ---------------------------------------------------------------------------
// Sun position (simplified VSOP87-derived algorithm)
// ---------------------------------------------------------------------------

/// Compute the Sun's apparent geocentric position.
/// Returns (azimuth_deg, elevation_deg) for the given observer and time.
/// Accuracy is ~0.01° which is far better than needed for a terminal display.
pub fn sun_position(time: &DateTime<Utc>, obs: Geodetic) -> (f64, f64) {
    let jd = coords::julian_date(time);
    let n = jd - 2451545.0; // days since J2000.0

    // --- Geometric Sun position (ecliptic coordinates) ---
    let g = (357.529 + 0.98560028 * n) % 360.0; // mean anomaly
    let q = (280.459 + 0.98564736 * n) % 360.0; // mean longitude

    let g_rad = g.to_radians();
    let lambda = q + 1.915 * g_rad.sin() + 0.020 * (2.0 * g_rad).sin(); // ecliptic longitude
    let lambda_rad = lambda.to_radians();

    // Obliquity of the ecliptic (mean, without nutation)
    let epsilon = (23.439 - 0.00000036 * n).to_radians();

    // --- Ecliptic → Equatorial ---
    let sin_l = lambda_rad.sin();
    let cos_l = lambda_rad.cos();
    let ra = (sin_l * epsilon.cos()).atan2(cos_l);
    let dec = (sin_l * epsilon.sin()).asin();

    // --- Equatorial → Horizontal ---
    equatorial_to_horizontal(ra, dec, jd, obs)
}

// ---------------------------------------------------------------------------
// Moon position (truncated ELP2000-82B / Meeus)
// ---------------------------------------------------------------------------

/// Compute the Moon's apparent geocentric position.
/// Returns (azimuth_deg, elevation_deg) for the given observer and time.
/// Accuracy is ~0.5°–1° which is fine for a terminal display.
pub fn moon_position(time: &DateTime<Utc>, obs: Geodetic) -> (f64, f64) {
    let jd = coords::julian_date(time);
    let n = jd - 2451545.0; // days since J2000.0

    let norm = |x: f64| x % 360.0;

    // --- Mean orbital elements (degrees) ---
    let lm = norm(218.3165 + 13.176396 * n); // mean longitude
    let mm = norm(134.9629 + 13.064993 * n); // mean anomaly (Moon)
    let ms = norm(357.5291 + 0.98560028 * n); // mean anomaly (Sun)
    let fm = norm(93.2720 + 13.229350 * n); // argument of latitude
    let ls = norm(280.459 + 0.98564736 * n); // mean longitude (Sun)
    let d = norm(lm - ls); // mean elongation

    let mm_rad = mm.to_radians();
    let d_rad = d.to_radians();
    let fm_rad = fm.to_radians();
    let ms_rad = ms.to_radians();

    // --- Ecliptic longitude (main perturbation terms) ---
    let lon = lm
        + 6.289 * mm_rad.sin()
        + 1.274 * (2.0 * d_rad - mm_rad).sin()
        + 0.658 * (2.0 * d_rad).sin()
        + 0.214 * (2.0 * mm_rad).sin()
        + 0.186 * ms_rad.sin()
        + 0.114 * (2.0 * fm_rad).sin();

    // --- Ecliptic latitude (main perturbation terms) ---
    let lat = 5.128 * fm_rad.sin()
        + 0.280 * (mm_rad + fm_rad).sin()
        + 0.278 * (mm_rad - fm_rad).sin()
        + 0.173 * (2.0 * d_rad - fm_rad).sin();

    let lambda_rad = lon.to_radians();
    let beta_rad = lat.to_radians();

    // Obliquity of the ecliptic
    let epsilon = (23.439 - 0.00000036 * n).to_radians();

    // --- Ecliptic → Equatorial ---
    let sin_l = lambda_rad.sin();
    let cos_l = lambda_rad.cos();
    let sin_b = beta_rad.sin();
    let cos_b = beta_rad.cos();

    // tan(RA) = [sin(λ) cos(ε) − tan(β) sin(ε)] / cos(λ)
    let ra = (sin_l * epsilon.cos() - (sin_b / cos_b) * epsilon.sin()).atan2(cos_l);

    // sin(Dec) = sin(β) cos(ε) + cos(β) sin(ε) sin(λ)
    let dec = (sin_b * epsilon.cos() + cos_b * epsilon.sin() * sin_l).asin();

    // --- Equatorial → Horizontal ---
    equatorial_to_horizontal(ra, dec, jd, obs)
}

// ---------------------------------------------------------------------------
// Shared conversion: Equatorial → Horizontal
// ---------------------------------------------------------------------------

/// Convert from right ascension/declination to az/el for a given observer.
/// `ra` and `dec` are in radians, `jd` is the Julian date, `obs` is the
/// observer's position.  Returns (azimuth_deg, elevation_deg).
fn equatorial_to_horizontal(ra: f64, dec: f64, jd: f64, obs: Geodetic) -> (f64, f64) {
    let gmst_val = coords::gmst(jd);
    let lmst = gmst_val + obs.lon.to_radians(); // local sidereal time
    let ha = lmst - ra; // hour angle

    let lat_rad = obs.lat.to_radians();

    // sin(el) = sin(φ)·sin(δ) + cos(φ)·cos(δ)·cos(H)
    let el = (lat_rad.sin() * dec.sin() + lat_rad.cos() * dec.cos() * ha.cos()).asin();

    // Azimuth measured from North, clockwise:
    //   az = atan2(−cos(δ)·sin(H),  sin(δ)·cos(φ) − cos(δ)·sin(φ)·cos(H))
    let az = (-dec.cos() * ha.sin()).atan2(
        dec.sin() * lat_rad.cos() - dec.cos() * lat_rad.sin() * ha.cos(),
    );

    let az_deg = (az.to_degrees() + 360.0) % 360.0;
    let el_deg = el.to_degrees();

    (az_deg, el_deg)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    /// Rough sanity: the Sun should be above the horizon during the day.
    #[test]
    fn test_sun_daytime() {
        // Chicago summer afternoon
        let time = Utc.with_ymd_and_hms(2026, 7, 2, 18, 0, 0).unwrap();
        let obs = Geodetic {
            lat: 41.8781,
            lon: -87.6298,
            alt: 0.0,
        };
        let (az, el) = sun_position(&time, obs);
        assert!(el > 0.0, "Sun should be above horizon in Chicago afternoon, got el={}", el);
        assert!((0.0..360.0).contains(&az));
    }

    /// Rough sanity: the Sun should be below the horizon at midnight.
    #[test]
    fn test_sun_nighttime() {
        let time = Utc.with_ymd_and_hms(2026, 7, 2, 4, 0, 0).unwrap();
        let obs = Geodetic {
            lat: 41.8781,
            lon: -87.6298,
            alt: 0.0,
        };
        let (_, el) = sun_position(&time, obs);
        assert!(el < 0.0, "Sun should be below horizon at night, got el={}", el);
    }

    /// The Moon's position should return sane az/el values.
    #[test]
    fn test_moon_sanity() {
        let time = Utc::now();
        let obs = Geodetic {
            lat: 41.8781,
            lon: -87.6298,
            alt: 0.0,
        };
        let (az, el) = moon_position(&time, obs);
        assert!((0.0..360.0).contains(&az), "az out of range: {}", az);
        assert!((-90.0..=90.0).contains(&el), "el out of range: {}", el);
    }
}