use anyhow::Result;
use sgp4::Elements;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct Tle {
    pub name: String,
    #[allow(dead_code)]
    pub line1: String,
    #[allow(dead_code)]
    pub line2: String,
    pub elements: Elements,
}

pub async fn fetch_tles() -> Result<Vec<Tle>> {
    let cache_dir = Path::new("cache");
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir)?;
    }

    let cache_file = cache_dir.join("tle_active.txt");
    let url = "https://celestrak.org/NORAD/elements/gp.php?GROUP=active&FORMAT=tle";

    let mut use_cache = false;
    if cache_file.exists()
        && let Ok(metadata) = fs::metadata(&cache_file)
        && let Ok(modified) = metadata.modified()
        && let Ok(age) = SystemTime::now().duration_since(modified)
        && age < Duration::from_secs(2 * 3600)
    {
        use_cache = true;
    }

    let content = if use_cache {
        info!("Using cached TLE data");
        fs::read_to_string(&cache_file)?
    } else {
        info!("Fetching fresh TLE data from Celestrak...");
        match reqwest::get(url).await {
            Ok(resp) => {
                let text = resp.text().await?;
                fs::write(&cache_file, &text)?;
                text
            }
            Err(e) => {
                warn!("Failed to fetch TLEs: {}. Falling back to cache if available.", e);
                if cache_file.exists() {
                    fs::read_to_string(&cache_file)?
                } else {
                    anyhow::bail!("No cache available and fetch failed.");
                }
            }
        }
    };

    Ok(parse_tles(&content))
}

fn parse_tles(data: &str) -> Vec<Tle> {
    let mut tles = Vec::new();
    let mut lines = data.lines().map(|l| l.trim()).filter(|l| !l.is_empty());

    while let Some(name) = lines.next() {
        if let Some(line1) = lines.next() {
            if let Some(line2) = lines.next() {
                if let Ok(elements) = Elements::from_tle(
                    Some(name.to_string()),
                    line1.as_bytes(),
                    line2.as_bytes(),
                ) {
                    tles.push(Tle {
                        name: name.to_string(),
                        line1: line1.to_string(),
                        line2: line2.to_string(),
                        elements,
                    });
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    tles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tles() {
        // Valid TLE sample (CALSPHERE 1 from Celestrak)
        let sample = "CALSPHERE 1\n\
                      1 00900U 64063C   26183.64242862  .00000615  00000+0  61698-3 0  9996\n\
                      2 00900  90.2223  72.1341 0023176 245.7721 146.4627 13.76639869 73552";
        let tles = parse_tles(sample);
        assert_eq!(tles.len(), 1);
        assert_eq!(tles[0].name, "CALSPHERE 1");
        assert_eq!(tles[0].elements.norad_id, 900);
    }
}
