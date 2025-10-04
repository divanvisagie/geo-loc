#[cfg(target_os = "macos")]
pub mod corelocation;

#[cfg(not(target_os = "macos"))]
pub mod corelocation {
    use crate::location::Location;
    use std::time::Duration;

    pub async fn get_current_location(
        _timeout: Duration,
        verbose: bool,
    ) -> Result<Location, Box<dyn std::error::Error>> {
        if verbose {
            eprintln!("geo-loc: CoreLocation provider requires macOS; falling back");
        }
        Err("CoreLocation provider is only available on macOS".into())
    }
}
