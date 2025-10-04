mod args;
mod location;
mod providers;

use args::{Args, Format, Provider};
use chrono::Utc;
use clap::Parser;
use location::Location;
use std::process;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let provider = resolve_provider(args.provider.clone());
    let timeout = Duration::from_secs(args.timeout);

    let location = match get_location(&provider, args.verbose, timeout).await {
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
            let acc = location
                .accuracy_m
                .map(|a| a.to_string())
                .unwrap_or_default();
            println!(
                "{},{},{},{},{}",
                location.latitude,
                location.longitude,
                acc,
                location.timestamp.to_rfc3339(),
                location.provider
            );
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
        Format::Plain => match location.accuracy_m {
            Some(acc) => println!(
                "{} {} (±{} m @ {})",
                location.latitude,
                location.longitude,
                (acc * 10.0).round() / 10.0,
                location.timestamp.to_rfc3339()
            ),
            None => println!(
                "{} {} {}",
                location.latitude,
                location.longitude,
                location.timestamp.to_rfc3339()
            ),
        },
    }
}

fn resolve_provider(requested: Provider) -> Provider {
    match requested {
        Provider::Auto => {
            #[cfg(target_os = "macos")]
            {
                Provider::Corelocation
            }
            #[cfg(not(target_os = "macos"))]
            {
                Provider::Ip
            }
        }
        other => other,
    }
}

async fn get_location(
    provider: &Provider,
    verbose: bool,
    timeout: Duration,
) -> Result<Location, Box<dyn std::error::Error>> {
    match provider {
        Provider::Ip => get_ip_location(verbose).await,
        Provider::Corelocation => {
            #[cfg(target_os = "macos")]
            {
                providers::corelocation::get_current_location(timeout, verbose).await
            }
            #[cfg(not(target_os = "macos"))]
            {
                providers::corelocation::get_current_location(timeout, verbose).await
            }
        }
        Provider::Auto => {
            #[cfg(target_os = "macos")]
            {
                match providers::corelocation::get_current_location(timeout, verbose).await {
                    Ok(loc) => Ok(loc),
                    Err(e) => {
                        if verbose {
                            eprintln!("geo-loc: CoreLocation error, falling back to IP: {}", e);
                        }
                        get_ip_location(verbose).await
                    }
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                match providers::corelocation::get_current_location(timeout, verbose).await {
                    Ok(loc) => Ok(loc),
                    Err(_) => get_ip_location(verbose).await,
                }
            }
        }
        Provider::Geoclue => Err("GeoClue provider not implemented".into()),
    }
}

async fn get_ip_location(verbose: bool) -> Result<Location, Box<dyn std::error::Error>> {
    if verbose {
        eprintln!("geo-loc: using IP-based geolocation");
    }
    let client = reqwest::Client::new();
    let resp: serde_json::Value = client
        .get("http://ip-api.com/json")
        .send()
        .await?
        .json()
        .await?;
    let lat = resp["lat"].as_f64().ok_or("Invalid latitude")?;
    let lon = resp["lon"].as_f64().ok_or("Invalid longitude")?;
    Ok(Location::new(lat, lon, None, "ip", Utc::now()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_resolution_defaults() {
        let resolved = resolve_provider(Provider::Auto);
        #[cfg(target_os = "macos")]
        assert!(matches!(resolved, Provider::Corelocation));
        #[cfg(not(target_os = "macos"))]
        assert!(matches!(resolved, Provider::Ip));
    }
}
