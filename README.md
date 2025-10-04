# geo-loc

A command-line utility to print the host's current geographic location in a pipe-friendly format. It queries native location services (CoreLocation on macOS, GeoClue on Linux) or falls back to IP-based geolocation, outputting in human-readable or machine-parseable formats.

Plain output now includes horizontal accuracy (if reported) and the provider's timestamp so you can tell how fresh the fix is at a glance.

## Why?

Many location tools are GUI-based or verbose. `geo-loc` follows Unix principles: silent operation, clean exit codes, and output suitable for scripting and pipelines. It's designed for automation, monitoring, or quick location checks without opening browsers or editing files.

## Installation

### From Crates.io (when published)
```bash
cargo install geo-loc
```

### From Source
```bash
git clone <repo>
cd geo-loc
cargo build --release
make install  # For system-wide install
# or make local-install  # For user install
```

## Usage

Get your location:
```bash
geo-loc
# Output: 58.5054 15.9724 (±12.3 m @ 2025-10-04T08:01:12Z)
```

For detailed options, formats, providers, and examples, see the man page:
```bash
man geo-loc  # After system install
# or
man ./geo-loc.1  # From source
```

## License

BSD 3-Clause