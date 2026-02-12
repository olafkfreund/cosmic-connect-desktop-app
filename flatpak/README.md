# COSMIC Connect - Flatpak Package

This directory contains the Flatpak packaging files for COSMIC Connect.

## Quick Start

### Prerequisites

```bash
# Install Flatpak and required runtimes
flatpak install flathub org.freedesktop.Platform//23.08 org.freedesktop.Sdk//23.08
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//23.08
```

### Generate Cargo Sources

Before building, generate the Cargo dependency sources:

```bash
# Clone flatpak-builder-tools
git clone https://github.com/flatpak/flatpak-builder-tools.git /tmp/flatpak-tools

# Generate sources from project root
python3 /tmp/flatpak-tools/cargo/flatpak-cargo-generator.py \
  Cargo.lock \
  -o flatpak/generated-sources.json
```

### Build and Install

```bash
# Build from project root
flatpak-builder --user --install --force-clean build-dir flatpak/io.github.olafkfreund.CosmicExtConnect.yml

# Run the application
flatpak run io.github.olafkfreund.CosmicExtConnect
```

## Files

| File | Description |
|------|-------------|
| `io.github.olafkfreund.CosmicExtConnect.yml` | Flatpak manifest (build configuration) |
| `io.github.olafkfreund.CosmicExtConnect.metainfo.xml` | AppStream metadata (app information) |
| `io.github.olafkfreund.CosmicExtConnect.desktop` | Desktop entry (manager application) |
| `io.github.olafkfreund.CosmicExtConnect.Applet.desktop` | Desktop entry (panel applet) |
| `FLATPAK.md` | Comprehensive build and submission guide |
| `README.md` | This file |
| `generated-sources.json` | Cargo dependencies (generated, not in git) |

## Documentation

See **[FLATPAK.md](FLATPAK.md)** for:

- Detailed build instructions
- Flathub submission process
- Sandboxing considerations
- Troubleshooting guide
- Maintenance procedures

## Components Included

The Flatpak package includes:

- **cosmic-ext-connect-manager** - Device management window
- **cosmic-ext-applet-connect** - COSMIC panel applet
- **cosmic-ext-connect-daemon** - Background service
- **cosmic-ext-messages-popup** - Web messaging interface
- **cosmic-messages** - CLI messaging utility
- **cosmic-ext-display-stream** - Display streaming library

## Daemon Limitations

The background daemon has limited functionality in the Flatpak sandbox. For full features (system-wide device integration, unrestricted file access), install the native package:

- **NixOS:** `nix profile install github:olafkfreund/cosmic-ext-connect-desktop-app`
- **Manual:** See main README.md

## Support

- **Issues:** https://github.com/olafkfreund/cosmic-ext-connect-desktop-app/issues
- **Documentation:** https://github.com/olafkfreund/cosmic-ext-connect-desktop-app/blob/main/docs/
- **Flathub Help:** https://docs.flatpak.org/

## License

GPL-3.0-or-later
