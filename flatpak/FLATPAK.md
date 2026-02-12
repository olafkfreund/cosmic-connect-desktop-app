# COSMIC Connect - Flatpak Packaging Guide

This guide covers building, testing, and submitting the COSMIC Connect Flatpak package.

## Overview

The Flatpak package provides a distribution-agnostic way to install COSMIC Connect on any Linux distribution. It includes:

- **cosmic-ext-connect-manager** - Standalone device management window
- **cosmic-ext-applet-connect** - COSMIC panel applet
- **cosmic-ext-connect-daemon** - Background service (limited in sandbox)
- **cosmic-ext-messages-popup** - Web messaging interface
- **cosmic-messages** - CLI messaging utility
- **cosmic-ext-display-stream** - Display streaming library

## Prerequisites

### Required Tools

```bash
# Install Flatpak and Flatpak Builder
sudo apt install flatpak flatpak-builder  # Debian/Ubuntu
sudo dnf install flatpak flatpak-builder  # Fedora
sudo pacman -S flatpak flatpak-builder    # Arch Linux

# Add Flathub repository
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

# Install required runtimes and SDKs
flatpak install flathub org.freedesktop.Platform//23.08
flatpak install flathub org.freedesktop.Sdk//23.08
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//23.08
```

### Important: Verify Checksums

**Before building for production**, verify all SHA256 checksums in the manifest:

```bash
# Example: Verify libxkbcommon
wget https://github.com/xkbcommon/libxkbcommon/archive/xkbcommon-1.6.0.tar.gz
sha256sum xkbcommon-1.6.0.tar.gz
# Compare with sha256 in manifest
```

Some checksums in the manifest are placeholders and **must be updated** before submitting to Flathub.

### Generate Cargo Sources

The manifest requires a `generated-sources.json` file containing all Cargo dependencies. Generate it using the flatpak-cargo-generator tool:

```bash
# Clone the cargo generator
git clone https://github.com/flatpak/flatpak-builder-tools.git

# Generate sources (run from project root)
cd /path/to/cosmic-connect-desktop-app
python3 ../flatpak-builder-tools/cargo/flatpak-cargo-generator.py \
  Cargo.lock \
  -o flatpak/generated-sources.json
```

This creates a JSON file mapping all Cargo dependencies to their source URLs and checksums.

## Building the Flatpak

### Local Development Build

Build the Flatpak locally for testing:

```bash
# From the project root
flatpak-builder --force-clean build-dir flatpak/io.github.olafkfreund.CosmicExtConnect.yml

# Install locally
flatpak-builder --user --install --force-clean build-dir flatpak/io.github.olafkfreund.CosmicExtConnect.yml
```

### Test the Application

```bash
# Run the manager
flatpak run io.github.olafkfreund.CosmicExtConnect

# Run with debugging enabled
flatpak run --devel --command=sh io.github.olafkfreund.CosmicExtConnect
# Then inside the sandbox:
cosmic-ext-connect-manager --verbose
```

### Export as Bundle

Create a distributable `.flatpak` bundle:

```bash
# Build and export
flatpak-builder --repo=repo --force-clean build-dir flatpak/io.github.olafkfreund.CosmicExtConnect.yml

# Create single-file bundle
flatpak build-bundle repo cosmic-connect.flatpak io.github.olafkfreund.CosmicExtConnect
```

Users can install the bundle with:

```bash
flatpak install cosmic-connect.flatpak
```

## Sandboxing Considerations

### Permissions Granted

The Flatpak manifest requests the following permissions:

| Permission | Reason |
|------------|--------|
| `--socket=wayland` | COSMIC Desktop is Wayland-native |
| `--socket=fallback-x11` | Compatibility with X11 applications |
| `--share=network` | Device discovery and communication |
| `--socket=session-bus` | D-Bus communication with daemon |
| `--socket=pipewire` | RemoteDesktop plugin and Camera as Webcam |
| `--filesystem=xdg-download:rw` | File sharing to Downloads folder |
| `--device=dri` | Hardware acceleration |
| `--allow=bluetooth` | RFCOMM Bluetooth support |

### Daemon Limitations

**Important:** The background daemon (`cosmic-ext-connect-daemon`) has limited functionality in the Flatpak sandbox:

- **D-Bus activation** works within the sandbox
- **Network discovery** works (UDP broadcast on port 1816)
- **File sharing** limited to XDG directories
- **System integration** features may be restricted

For full functionality, users should install the native package:

```bash
# NixOS (recommended)
nix profile install github:olafkfreund/cosmic-connect-desktop-app

# Manual installation (see README.md)
cargo build --release
sudo install target/release/cosmic-ext-connect-daemon /usr/local/bin/
```

### Testing Sandbox Restrictions

Test specific sandbox scenarios:

```bash
# Test network access
flatpak run --command=cosmic-ext-connect-daemon io.github.olafkfreund.CosmicExtConnect --test-network

# Test file access
flatpak run --command=sh io.github.olafkfreund.CosmicExtConnect
ls ~/Downloads  # Should work
ls ~/Documents  # Should work
ls /etc         # Should fail (no host access)
```

## Flathub Submission

### Preparation Checklist

Before submitting to Flathub, ensure:

- [ ] `generated-sources.json` is up to date
- [ ] AppStream metadata is valid
- [ ] Screenshots are available (1920x1080 recommended)
- [ ] All tests pass locally
- [ ] No hardcoded user paths
- [ ] License files included
- [ ] Desktop files validate with `desktop-file-validate`
- [ ] AppStream validates with `appstreamcli validate`

### Validation

```bash
# Validate desktop file
desktop-file-validate flatpak/io.github.olafkfreund.CosmicExtConnect.desktop

# Validate AppStream metadata
appstreamcli validate flatpak/io.github.olafkfreund.CosmicExtConnect.metainfo.xml

# Check for common Flatpak issues
flatpak run org.flatpak.Builder --show-manifest flatpak/io.github.olafkfreund.CosmicExtConnect.yml
```

### Submit to Flathub

1. **Fork the Flathub repository:**

   ```bash
   # Fork on GitHub: https://github.com/flathub/flathub
   git clone https://github.com/YOUR_USERNAME/flathub.git
   cd flathub
   ```

2. **Create application branch:**

   ```bash
   git checkout -b add-cosmic-connect
   mkdir io.github.olafkfreund.CosmicExtConnect
   ```

3. **Copy manifest and metadata:**

   ```bash
   cp /path/to/cosmic-connect/flatpak/io.github.olafkfreund.CosmicExtConnect.yml io.github.olafkfreund.CosmicExtConnect/
   cp /path/to/cosmic-connect/flatpak/io.github.olafkfreund.CosmicExtConnect.metainfo.xml io.github.olafkfreund.CosmicExtConnect/
   cp /path/to/cosmic-connect/flatpak/generated-sources.json io.github.olafkfreund.CosmicExtConnect/
   cp /path/to/cosmic-connect/flatpak/*.desktop io.github.olafkfreund.CosmicExtConnect/
   ```

4. **Add flathub.json (required):**

   ```bash
   cat > io.github.olafkfreund.CosmicExtConnect/flathub.json << 'EOF'
   {
     "only-arches": ["x86_64", "aarch64"]
   }
   EOF
   ```

5. **Create pull request:**

   ```bash
   git add io.github.olafkfreund.CosmicExtConnect
   git commit -m "Add io.github.olafkfreund.CosmicExtConnect"
   git push origin add-cosmic-connect
   ```

   Then create a pull request on GitHub from your fork to `flathub/flathub`.

6. **Respond to review feedback:**

   Flathub maintainers will review your submission. Common feedback:

   - Update checksums/hashes
   - Reduce permissions
   - Fix AppStream metadata
   - Add missing dependencies
   - Improve descriptions

### Post-Submission

Once merged, your application will be available on Flathub:

```bash
flatpak install flathub io.github.olafkfreund.CosmicExtConnect
```

## Maintenance

### Updating the Flatpak

When releasing a new version:

1. **Update the manifest:**

   ```yaml
   # In io.github.olafkfreund.CosmicExtConnect.yml
   sources:
     - type: git
       url: https://github.com/olafkfreund/cosmic-connect-desktop-app.git
       tag: v0.2.0  # Update version
       commit: abc123def456  # Update commit hash
   ```

2. **Regenerate Cargo sources:**

   ```bash
   python3 flatpak-cargo-generator.py Cargo.lock -o flatpak/generated-sources.json
   ```

3. **Update AppStream metadata:**

   ```xml
   <!-- Add new release in metainfo.xml -->
   <release version="0.2.0" date="2026-03-01">
     <description>
       <p>What's new in version 0.2.0</p>
     </description>
   </release>
   ```

4. **Submit update to Flathub:**

   ```bash
   cd flathub/io.github.olafkfreund.CosmicExtConnect
   # Update files
   git commit -am "Update to version 0.2.0"
   git push origin update-0.2.0
   # Create PR
   ```

### Monitoring Build Status

- **Build logs:** https://buildbot.flathub.org
- **Build status:** Check your Flathub PR for automated build results
- **User reports:** Monitor GitHub issues for Flatpak-specific bugs

## Troubleshooting

### Common Build Issues

**Issue: Missing Cargo dependencies**

```
Error: Failed to fetch cargo dependency
```

**Solution:** Regenerate `generated-sources.json`:

```bash
python3 flatpak-cargo-generator.py Cargo.lock -o flatpak/generated-sources.json
```

---

**Issue: Permission denied for D-Bus**

```
Error: Could not connect to session bus
```

**Solution:** Add D-Bus permission in manifest:

```yaml
finish-args:
  - --socket=session-bus
  - --own-name=io.github.olafkfreund.CosmicExtConnect
```

---

**Issue: PipeWire not found**

```
Error: PipeWire library not found
```

**Solution:** Ensure PipeWire module is included and built before cosmic-connect module.

---

**Issue: WebKitGTK build timeout**

WebKitGTK takes a long time to build. For local testing, use pre-built version:

```yaml
# Remove webkitgtk module and add runtime extension
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable
  - org.gnome.Sdk  # Includes WebKitGTK
```

### Getting Help

- **Flathub Matrix:** #flathub:matrix.org
- **Flatpak IRC:** #flatpak on OFTC
- **Documentation:** https://docs.flatpak.org/
- **COSMIC Connect Issues:** https://github.com/olafkfreund/cosmic-connect-desktop-app/issues

## Architecture Notes

### Build Process

The Flatpak build process:

1. **Setup Rust toolchain** from SDK extension
2. **Build dependencies** (libxkbcommon, PipeWire, WebKitGTK, GStreamer)
3. **Fetch Cargo sources** offline from `generated-sources.json`
4. **Build workspace** with all components
5. **Install binaries** to `/app/bin/`
6. **Install desktop integration** files
7. **Create D-Bus services** for activation

### Runtime Dependencies

The freedesktop runtime (23.08) provides:

- Wayland/X11 support
- OpenSSL
- D-Bus
- Basic system libraries

Additional libraries built from source:

- libxkbcommon (Wayland keyboard handling)
- PipeWire (screen sharing, webcam)
- WebKitGTK (web messaging)
- GStreamer (media streaming)

### Multi-Architecture Support

The manifest supports:

- **x86_64** (primary)
- **aarch64** (ARM 64-bit)

Cross-compilation handled by Flatpak Builder automatically when building for Flathub.

## References

- [Flatpak Documentation](https://docs.flatpak.org/)
- [Flathub Submission Guidelines](https://github.com/flathub/flathub/wiki/App-Submission)
- [AppStream Specification](https://www.freedesktop.org/software/appstream/docs/)
- [Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/)
- [COSMIC Connect Repository](https://github.com/olafkfreund/cosmic-connect-desktop-app)

## License

This packaging is licensed under GPL-3.0-or-later, consistent with the COSMIC Connect project.
