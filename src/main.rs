mod args;

use clap::Parser;
use args::{Args, Format, Provider};
use serde::{Deserialize, Serialize};
use std::process;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
struct Location {
    latitude: f64,
    longitude: f64,
    accuracy_m: Option<f64>,
    provider: String,
    timestamp: DateTime<Utc>,
}

impl Location {
    fn new(lat: f64, lon: f64, acc: Option<f64>, provider: &str) -> Self {
        Self {
            latitude: lat,
            longitude: lon,
            accuracy_m: acc,
            provider: provider.to_string(),
            timestamp: Utc::now(),
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let provider = match args.provider {
        Provider::Auto => Provider::Ip, // Default to IP for now
        p => p,
    };

    let location = match get_location(&provider, args.verbose).await {
        Ok(loc) => loc,
        Err(e) => {
            if args.verbose {
                eprintln!("geo-loc: {}", e);
            }
            process::exit(1);
        }
    };

    match args.format {
        Format::Json => println!("{}", serde_json::to_string(&location).unwrap()),
        Format::Csv => {
            let acc = location.accuracy_m.map(|a| a.to_string()).unwrap_or_default();
            println!("{},{},{},{},{}", location.latitude, location.longitude, acc, location.timestamp.to_rfc3339(), location.provider);
        }
        Format::Env => {
            println!("LAT={}", location.latitude);
            println!("LON={}", location.longitude);
            if let Some(acc) = location.accuracy_m {
                println!("ACC={}", acc);
            }
            println!("PROVIDER={}", location.provider);
            println!("TS={}", location.timestamp.to_rfc3339());
        }
        Format::Plain => println!("{} {}", location.latitude, location.longitude),
    }
}

async fn get_location(provider: &Provider, verbose: bool) -> Result<Location, Box<dyn std::error::Error>> {
    match provider {
        Provider::Ip => get_ip_location(verbose).await,
        _ => Err("Provider not implemented".into()),
    }
}

async fn get_ip_location(verbose: bool) -> Result<Location, Box<dyn std::error::Error>> {
    if verbose {
        eprintln!("geo-loc: using IP-based geolocation");
    }
    let client = reqwest::Client::new();
    let resp: serde_json::Value = client.get("http://ip-api.com/json").send().await?.json().await?;
    let lat = resp["lat"].as_f64().ok_or("Invalid latitude")?;
    let lon = resp["lon"].as_f64().ok_or("Invalid longitude")?;
    Ok(Location::new(lat, lon, None, "ip"))
}