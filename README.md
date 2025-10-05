# geo-loc

A simple command-line tool that returns your current geographic location in a pipe-friendly format.

## Features

- **GeoClue Integration**: Uses Linux native location services for accurate positioning
- **IP Fallback**: Automatically falls back to IP-based geolocation when GeoClue is unavailable
- **Simple Output**: Returns coordinates as "latitude longitude" (space-separated)
- **Proper Exit Codes**: Unix-compliant exit codes for scripting
- **Fast**: 5-second timeout with immediate fallback

## Installation

### From Source

1. **Build the binary:**
   ```bash
   cargo build --release
   ```

2. **Install using the provided script:**
   ```bash
   ./install.sh
   ```

### Manual Installation

1. **Install binary:**
   ```bash
   sudo cp target/release/geo-loc /usr/local/bin/
   sudo chmod +x /usr/local/bin/geo-loc
   ```

2. **Install desktop file (REQUIRED for GeoClue):**
   ```bash
   sudo cp resources/geo-loc.desktop /usr/share/applications/
   sudo update-desktop-database
   ```

## Usage

### Basic Usage

```bash
# Get current location
$ geo-loc
55.582917688169935 12.9199

# Check exit code
$ geo-loc && echo "Success: $(geo-loc)"
```

### Use in Scripts

```bash
# Store coordinates
LAT_LON=$(geo-loc)
if [ $? -eq 0 ]; then
    echo "Current location: $LAT_LON"
    LAT=$(echo $LAT_LON | cut -d' ' -f1)
    LON=$(echo $LAT_LON | cut -d' ' -f2)
else
    echo "Location unavailable"
fi

# Pipe-friendly processing
geo-loc | awk '{printf "Latitude: %.4f\nLongitude: %.4f\n", $1, $2}'
```

## Requirements

### Runtime Dependencies

- **Linux system** with D-Bus support
- **GeoClue 2** (optional, for accurate location):
  ```bash
  sudo apt install geoclue-2.0
  ```
- **Network access** (for IP fallback)

### Build Dependencies

- **Rust 1.70+**
- **D-Bus development libraries**:
  ```bash
  sudo apt install libdbus-1-dev
  ```

## Location Services Setup

### For GeoClue (Recommended)

1. **Install GeoClue:**
   ```bash
   sudo apt install geoclue-2.0
   ```

2. **Enable location services in GNOME:**
   - Open Settings (`gnome-control-center`)
   - Go to Privacy & Security → Location Services  
   - Enable Location Services

3. **Verify desktop file is installed:**
   ```bash
   ls -la /usr/share/applications/geo-loc.desktop
   ```

### Alternative: Manual GeoClue Configuration

If the GUI method doesn't work, you can manually configure GeoClue:

```bash
sudo tee -a /etc/geoclue/geoclue.conf << EOF

[geo-loc]
allowed=true
system=false
users=
EOF
```

## Exit Codes

- `0` - Success, coordinates printed to stdout
- `1` - Generic error (network issues, invalid response)  
- `70` - Location service unavailable (GeoClue not installed/running)
- `77` - Permission denied (location access disabled)

## Providers

### GeoClue (Primary)
- Uses system location services
- Accuracy: ~10-100 meters (depends on available sources)
- Sources: WiFi, GPS, cell towers, IP geolocation
- Requires: Desktop file installation and location permissions

### IP-based (Fallback)  
- Uses `ip-api.com` service
- Accuracy: City-level (~1-50km)
- No permissions required
- Requires: Internet connection

## Troubleshooting

### GeoClue Issues

```bash
# Check if GeoClue is running
systemctl status geoclue

# Check if service is accessible via D-Bus
busctl list | grep -i geoclue

# Test D-Bus manually
busctl call org.freedesktop.GeoClue2 /org/freedesktop/GeoClue2/Manager org.freedesktop.GeoClue2.Manager GetClient
```

### Permission Issues

```bash
# Verify desktop file exists
ls -la /usr/share/applications/geo-loc.desktop

# Check GeoClue configuration
sudo grep -A3 "\[geo-loc\]" /etc/geoclue/geoclue.conf
```

### Network Issues

```bash
# Test IP provider directly
curl -s http://ip-api.com/json | jq '.lat, .lon'
```

## Examples

### Weather Integration
```bash
COORDS=$(geo-loc)
LAT=$(echo $COORDS | cut -d' ' -f1)  
LON=$(echo $COORDS | cut -d' ' -f2)
curl "https://api.weather.gov/points/$LAT,$LON"
```

### Reverse Geocoding
```bash
geo-loc | xargs -I{} curl -s "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=json"
```

### Distance Calculation
```bash
# Distance to a landmark (example: Times Square NYC)
COORDS=$(geo-loc)
echo "$COORDS 40.7580 -73.9855" | awk '{
    lat1=$1; lon1=$2; lat2=$3; lon2=$4
    # Haversine formula would go here
    print "Current:", lat1, lon1
    print "Target:", lat2, lon2
}'
```

## Building

```bash
# Development build
cargo build

# Release build  
cargo build --release

# Run tests
cargo test

# Check code
cargo clippy
cargo fmt
```

## License

BSD-3-Clause

## Contributing

1. Ensure the tool maintains Unix philosophy (do one thing well)
2. Preserve the simple output format for pipe compatibility
3. Add tests for new functionality
4. Follow existing error code conventions

---

**Note**: This tool prioritizes accuracy and reliability over speed. The 5-second timeout ensures we get the best available location data before falling back to IP-based positioning.