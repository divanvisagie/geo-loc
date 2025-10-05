//! Simple geographic location CLI tool
//!
//! Tries GeoClue first, falls back to IP geolocation

use std::process;

#[tokio::main]
async fn main() {
    match get_location().await {
        Ok((lat, lon)) => {
            println!("{} {}", lat, lon);
        }
        Err(e) => {
            eprintln!("geo-loc: {}", e.message);
            process::exit(e.code);
        }
    }
}

async fn get_location() -> Result<(f64, f64), Error> {
    // Try GeoClue first
    match try_geoclue().await {
        Ok(coords) => Ok(coords),
        Err(e) => {
            // Show helpful message for permission errors, then fall back to IP
            if e.code == 77 {
                eprintln!("geo-loc: {}", e.message);
                eprintln!("geo-loc: falling back to IP-based location...");
            }
            try_ip_location().await
        }
    }
}

async fn try_geoclue() -> Result<(f64, f64), Error> {
    use zbus::Connection;

    // Connect to system bus
    let connection = Connection::system()
        .await
        .map_err(|_| Error::service_unavailable())?;

    // Get manager proxy
    let manager = ManagerProxy::new(&connection)
        .await
        .map_err(|_| Error::service_unavailable())?;

    // Get client
    let client_path = manager.get_client().await.map_err(|e| match e {
        zbus::Error::MethodError(name, _, _) if name.contains("NotAuthorized") => {
            Error::permission_denied()
        }
        _ => Error::service_unavailable(),
    })?;

    // Set up client
    let client = ClientProxy::builder(&connection)
        .path(&client_path)
        .map_err(|_| Error::service_unavailable())?
        .build()
        .await
        .map_err(|_| Error::service_unavailable())?;

    // Set desktop ID property
    client
        .set_desktop_id("geo-loc")
        .await
        .map_err(|_| Error::service_unavailable())?;

    // Set accuracy level property
    client
        .set_requested_accuracy_level(4)
        .await // Street level
        .map_err(|_| Error::service_unavailable())?;

    // Start location service
    client.start().await.map_err(|e| match e {
        zbus::Error::MethodError(name, _, _) if name.contains("NotAuthorized") => {
            Error::permission_denied()
        }
        _ => Error::service_unavailable(),
    })?;

    // Get location with timeout
    let location_path = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        // Poll for location
        for _ in 0..10 {
            if let Ok(path) = client.location().await {
                if !path.as_str().is_empty() {
                    return Ok(path);
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        Err(Error::timeout())
    })
    .await
    .map_err(|_| Error::timeout())??;

    // Read coordinates
    let location = LocationProxy::builder(&connection)
        .path(&location_path)
        .map_err(|_| Error::service_unavailable())?
        .build()
        .await
        .map_err(|_| Error::service_unavailable())?;

    let lat = location
        .latitude()
        .await
        .map_err(|_| Error::service_unavailable())?;
    let lon = location
        .longitude()
        .await
        .map_err(|_| Error::service_unavailable())?;

    // Stop client
    let _ = client.stop().await;

    Ok((lat, lon))
}

async fn try_ip_location() -> Result<(f64, f64), Error> {
    let client = reqwest::Client::new();
    let response: serde_json::Value = client
        .get("http://ip-api.com/json")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|_| Error::network())?
        .json()
        .await
        .map_err(|_| Error::network())?;

    let lat = response["lat"].as_f64().ok_or_else(|| Error::network())?;
    let lon = response["lon"].as_f64().ok_or_else(|| Error::network())?;

    Ok((lat, lon))
}

// Simple error type
#[derive(Debug)]
struct Error {
    message: String,
    code: i32,
}

impl Error {
    fn permission_denied() -> Self {
        Self {
            message: "permission denied - location services disabled\n\nTo enable location access:\n1. Open GNOME Settings (gnome-control-center)\n2. Go to Privacy & Security → Location Services\n3. Enable Location Services\n4. Ensure geo-loc.desktop is installed in /usr/share/applications/\n\nAlternatively, falling back to IP-based location...".into(),
            code: 77,
        }
    }

    fn service_unavailable() -> Self {
        Self {
            message: "location service unavailable - install geoclue-2.0 package".into(),
            code: 70,
        }
    }

    fn timeout() -> Self {
        Self {
            message: "timeout waiting for location".into(),
            code: 1,
        }
    }

    fn network() -> Self {
        Self {
            message: "network error - check internet connection".into(),
            code: 1,
        }
    }
}

// D-Bus proxy traits (minimal)
#[zbus::dbus_proxy(
    interface = "org.freedesktop.GeoClue2.Manager",
    default_service = "org.freedesktop.GeoClue2",
    default_path = "/org/freedesktop/GeoClue2/Manager"
)]
trait Manager {
    fn get_client(&self) -> zbus::Result<zvariant::OwnedObjectPath>;
}

#[zbus::dbus_proxy(
    interface = "org.freedesktop.GeoClue2.Client",
    default_service = "org.freedesktop.GeoClue2"
)]
trait Client {
    fn start(&self) -> zbus::Result<()>;
    fn stop(&self) -> zbus::Result<()>;

    #[dbus_proxy(property)]
    fn location(&self) -> zbus::Result<zvariant::OwnedObjectPath>;

    #[dbus_proxy(property)]
    fn desktop_id(&self) -> zbus::Result<String>;
    #[dbus_proxy(property)]
    fn set_desktop_id(&self, value: &str) -> zbus::Result<()>;

    #[dbus_proxy(property)]
    fn requested_accuracy_level(&self) -> zbus::Result<u32>;
    #[dbus_proxy(property)]
    fn set_requested_accuracy_level(&self, value: u32) -> zbus::Result<()>;
}

#[zbus::dbus_proxy(
    interface = "org.freedesktop.GeoClue2.Location",
    default_service = "org.freedesktop.GeoClue2"
)]
trait Location {
    #[dbus_proxy(property)]
    fn latitude(&self) -> zbus::Result<f64>;

    #[dbus_proxy(property)]
    fn longitude(&self) -> zbus::Result<f64>;
}
