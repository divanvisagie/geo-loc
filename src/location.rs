use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_m: Option<f64>,
    pub provider: String,
    pub timestamp: DateTime<Utc>,
}

impl Location {
    pub fn new(
        lat: f64,
        lon: f64,
        acc: Option<f64>,
        provider: &str,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            latitude: lat,
            longitude: lon,
            accuracy_m: acc,
            provider: provider.to_string(),
            timestamp,
        }
    }
}
