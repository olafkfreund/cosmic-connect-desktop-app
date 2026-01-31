# NixOS Packaging

This directory contains NixOS packaging files for COSMIC Connect.

## Files

- **package.nix** - Package derivation for building cosmic-connect
- **module.nix** - NixOS module with configuration options
- **tests.nix** - NixOS VM tests for the package and module
- **README.md** - This file

## Quick Start

### Using the Flake (Recommended)

Add to your `flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    cosmic-connect.url = "github:olafkfreund/cosmic-connect-desktop-app";
  };

  outputs = { self, nixpkgs, cosmic-connect }: {
    nixosConfigurations.your-hostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        cosmic-connect.nixosModules.default
        {
          services.cosmic-connect = {
            enable = true;
            openFirewall = true;
          };
        }
      ];
    };
  };
}
```

Then rebuild:

```bash
sudo nixos-rebuild switch --flake .#your-hostname
```

### Using the Overlay

```nix
{
  inputs.cosmic-connect.url = "github:olafkfreund/cosmic-connect-desktop-app";

  outputs = { nixpkgs, cosmic-connect, ... }: {
    nixosConfigurations.your-hostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [{
        nixpkgs.overlays = [ cosmic-connect.overlays.default ];
        environment.systemPackages = [ pkgs.cosmic-connect ];
      }];
    };
  };
}
```

### Traditional NixOS Configuration

If not using flakes, copy the files to your configuration:

```nix
{ config, pkgs, ... }:

{
  imports = [ /path/to/cosmic-connect-desktop-app/nix/module.nix ];

  services.cosmic-connect = {
    enable = true;
    openFirewall = true;
  };
}
```

## Module Options Reference

### Basic Options

```nix
services.cosmic-connect = {
  # Enable the service
  enable = true;

  # Open firewall ports (1814-1864 TCP/UDP for CConnect protocol)
  openFirewall = true;

  # Package to use (normally auto-detected)
  package = pkgs.cosmic-connect;
};
```

### Daemon Configuration

```nix
services.cosmic-connect.daemon = {
  # Enable daemon service
  enable = true;

  # Auto-start on login
  autoStart = true;

  # Logging level: "error" | "warn" | "info" | "debug" | "trace"
  logLevel = "info";

  # Custom settings (written to daemon.toml)
  settings = {
    discovery = {
      broadcast_interval = 5000;
      listen_port = 1816;
    };
  };
};
```

### Plugin Configuration

All plugins can be enabled/disabled individually:

```nix
services.cosmic-connect.plugins = {
  # Core Communication
  ping = true;           # Connectivity testing
  battery = true;        # Battery monitoring
  notification = true;   # Notification mirroring
  share = true;          # File/text/URL sharing
  clipboard = true;      # Clipboard sync
  telephony = true;      # Call & SMS notifications
  contacts = false;      # Contact sync (opt-in)

  # Control
  mpris = true;          # Media player control
  remoteinput = true;    # Mouse & keyboard control
  runcommand = false;    # Remote command execution (security: opt-in)
  findmyphone = true;    # Ring device
  presenter = false;     # Presentation mode (opt-in)

  # System
  systemmonitor = true;  # CPU/RAM stats sharing
  lock = true;           # Remote lock/unlock
  wol = true;            # Wake-on-LAN
  screenshot = true;     # Screen capture

  # Advanced (security-sensitive, disabled by default)
  remotedesktop = false; # VNC screen sharing
};
```

### Applet Configuration

```nix
services.cosmic-connect.applet = {
  # Enable COSMIC panel applet
  enable = true;
};
```

### Security Options

```nix
services.cosmic-connect.security = {
  # Certificate storage directory
  certificateDirectory = "~/.config/cosmic-connect/certs";

  # Trust on first pair (disable for enhanced security)
  trustOnFirstPair = true;
};
```

### Storage Options

```nix
services.cosmic-connect.storage = {
  # Where received files are stored
  downloadDirectory = "~/Downloads";

  # Base data directory
  dataDirectory = "~/.local/share/cosmic-connect";
};
```

## Complete Example

```nix
{ config, pkgs, ... }:

{
  services.cosmic-connect = {
    enable = true;
    openFirewall = true;

    daemon = {
      enable = true;
      autoStart = true;
      logLevel = "info";
    };

    applet.enable = true;

    plugins = {
      # Enable core features
      battery = true;
      clipboard = true;
      notification = true;
      share = true;
      mpris = true;
      ping = true;
      findmyphone = true;
      lock = true;

      # Disable security-sensitive plugins
      runcommand = false;
      remotedesktop = false;
    };

    security = {
      trustOnFirstPair = false;  # Enhanced security
    };

    storage = {
      downloadDirectory = "~/Downloads/CosmicConnect";
    };
  };
}
```

## Network Configuration

### Firewall Ports

COSMIC Connect uses the **CConnect protocol** (port 1816) which runs side-by-side with KDE Connect:

| Protocol | Ports | Purpose |
|----------|-------|---------|
| CConnect Discovery | 1814-1864 TCP/UDP | Device discovery |
| File Transfer | 1739-1764 TCP | File sharing |

The `openFirewall = true` option automatically configures these ports.

### Manual Firewall Configuration

If you need manual control:

```nix
networking.firewall = {
  allowedTCPPortRanges = [
    { from = 1814; to = 1864; }  # CConnect discovery
    { from = 1739; to = 1764; }  # File transfer
  ];
  allowedUDPPortRanges = [
    { from = 1814; to = 1864; }  # CConnect discovery
  ];
};
```

## Building the Package

### From Flake

```bash
# Build the package
nix build .#default

# Install to user profile
nix profile install .#default

# Run directly without installing
nix run .#default

# Enter development shell
nix develop
```

### Development Shell Features

The dev shell provides:
- Rust toolchain with rust-analyzer and clippy
- All build dependencies (libcosmic, GStreamer, PipeWire, etc.)
- Development tools (just, git, etc.)
- Automatic PKG_CONFIG_PATH configuration
- Verification of critical dependencies

```bash
nix develop
# Output shows:
# 🚀 COSMIC Connect Development Environment
# ✓ dbus-1 found
# ✓ openssl found
# ✓ gstreamer found
```

## Running Tests

### All Tests

```bash
nix flake check
```

### Specific Tests

```bash
# Package build test
nix build .#checks.x86_64-linux.package-build

# Basic module test
nix build .#checks.x86_64-linux.module-basic

# Custom config test
nix build .#checks.x86_64-linux.module-custom-config

# Two machines communication test
nix build .#checks.x86_64-linux.two-machines

# Security hardening test
nix build .#checks.x86_64-linux.security-test
```

### Available Test Cases

1. **package-build** - Verifies package builds correctly
2. **module-basic** - Basic module configuration
3. **module-custom-config** - Custom settings and plugins
4. **module-no-firewall** - Firewall disabled configuration
5. **two-machines** - Device discovery between machines
6. **plugin-test** - Plugin enable/disable functionality
7. **service-recovery** - Service restart on failure
8. **security-test** - Systemd security hardening

## Troubleshooting

### Build Failures

```bash
# Check build logs
nix log .#default

# Build with verbose output
nix build .#default --print-build-logs

# Check for missing dependencies
nix develop --command bash -c "cargo check 2>&1 | head -50"
```

### Service Issues

```bash
# Check daemon status
systemctl --user status cosmic-connect-daemon

# View daemon logs
journalctl --user -u cosmic-connect-daemon -f

# Restart daemon
systemctl --user restart cosmic-connect-daemon
```

### Module Errors

```bash
# Check module options
nix eval .#nixosModules.default.options --json | jq

# Test module configuration (dry run)
nixos-rebuild dry-build --flake .#your-hostname
```

### Common Issues

| Issue | Solution |
|-------|----------|
| "Device not discovered" | Check `openFirewall = true` and that both devices are on same network |
| "Pairing failed" | Ensure certificates directory is writable |
| "Plugin not working" | Verify plugin is enabled in config |
| "DBus error" | Check `services.dbus.packages` includes the package |

## Resources

- [COSMIC Connect Repository](https://github.com/olafkfreund/cosmic-connect-desktop-app)
- [NixOS Manual - Packaging](https://nixos.org/manual/nixpkgs/stable/#chap-stdenv)
- [NixOS Manual - Modules](https://nixos.org/manual/nixos/stable/#sec-writing-modules)
- [Rust in Nixpkgs](https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md)

## License

GPL-3.0-or-later - Same as the main project
