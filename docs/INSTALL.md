# Installation Guide

This guide covers installation of COSMIC KDE Connect on various Linux distributions.

## Table of Contents

- [Prerequisites](#prerequisites)
- [NixOS Installation](#nixos-installation)
- [Ubuntu/Debian Installation](#ubuntudebian-installation)
- [Fedora Installation](#fedora-installation)
- [Arch Linux Installation](#arch-linux-installation)
- [From Source](#from-source)
- [Firewall Configuration](#firewall-configuration)
- [Mobile App Installation](#mobile-app-installation)

## Prerequisites

### System Requirements

- COSMIC Desktop Environment (Alpha 2 or later)
- Linux kernel 5.10+
- Network connectivity (WiFi or Ethernet)
- At least 100MB free disk space

### Mobile Device

- **Android**: Version 4.4+ (KitKat or later)
- **iOS**: Version 14.0+ (iPhone, iPad)

## NixOS Installation

### Using Flakes (Recommended)

Add to your `flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    cosmic-applet-kdeconnect.url = "github:olafkfreund/cosmic-applet-kdeconnect";
  };

  outputs = { self, nixpkgs, cosmic-applet-kdeconnect }: {
    nixosConfigurations.your-hostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        {
          environment.systemPackages = [
            cosmic-applet-kdeconnect.packages.x86_64-linux.default
          ];

          # Firewall configuration
          networking.firewall = {
            allowedTCPPortRanges = [
              { from = 1714; to = 1764; }
            ];
            allowedUDPPortRanges = [
              { from = 1714; to = 1764; }
            ];
          };

          # Enable the daemon service
          systemd.user.services.kdeconnect-daemon = {
            description = "KDE Connect Daemon";
            wantedBy = [ "default.target" ];
            serviceConfig = {
              ExecStart = "${cosmic-applet-kdeconnect.packages.x86_64-linux.default}/bin/kdeconnect-daemon";
              Restart = "on-failure";
            };
          };
        }
      ];
    };
  };
}
```

### Traditional NixOS Configuration

Add to your `configuration.nix`:

```nix
{
  environment.systemPackages = with pkgs; [
    cosmic-applet-kdeconnect
  ];

  networking.firewall = {
    allowedTCPPortRanges = [
      { from = 1714; to = 1764; }
    ];
    allowedUDPPortRanges = [
      { from = 1714; to = 1764; }
    ];
  };
}
```

Then rebuild your system:

```bash
sudo nixos-rebuild switch
```

## Ubuntu/Debian Installation

### Install Dependencies

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  pkg-config \
  libxkbcommon-dev \
  libwayland-dev \
  libwayland-protocols \
  libgl1-mesa-dev \
  libglvnd-dev \
  libpixman-1-dev \
  libinput-dev \
  libxcb1-dev \
  libxcb-util-dev \
  libxcb-image0-dev \
  libdrm-dev \
  libfontconfig1-dev \
  libfreetype6-dev \
  libudev-dev \
  libdbus-1-dev \
  libpulse-dev \
  libexpat1-dev \
  libssl-dev \
  curl \
  git
```

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Install Just

```bash
cargo install just
```

### Build and Install

```bash
# Clone the repository
git clone https://github.com/olafkfreund/cosmic-applet-kdeconnect.git
cd cosmic-applet-kdeconnect

# Build release binaries
cargo build --release

# Install system-wide
sudo install -Dm755 target/release/cosmic-applet-kdeconnect \
  /usr/local/bin/cosmic-applet-kdeconnect
sudo install -Dm755 target/release/kdeconnect-daemon \
  /usr/local/bin/kdeconnect-daemon

# Install systemd service
mkdir -p ~/.config/systemd/user
cp kdeconnect-daemon/kdeconnect-daemon.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now kdeconnect-daemon
```

### Configure Firewall (UFW)

```bash
sudo ufw allow 1714:1764/tcp
sudo ufw allow 1714:1764/udp
sudo ufw reload
```

## Fedora Installation

### Install Dependencies

```bash
sudo dnf install -y \
  gcc \
  gcc-c++ \
  pkg-config \
  libxkbcommon-devel \
  wayland-devel \
  wayland-protocols-devel \
  mesa-libGL-devel \
  libglvnd-devel \
  pixman-devel \
  libinput-devel \
  libxcb-devel \
  xcb-util-devel \
  xcb-util-image-devel \
  libdrm-devel \
  fontconfig-devel \
  freetype-devel \
  systemd-devel \
  dbus-devel \
  pulseaudio-libs-devel \
  expat-devel \
  openssl-devel \
  curl \
  git
```

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Install Just

```bash
cargo install just
```

### Build and Install

Follow the same build steps as Ubuntu/Debian above.

### Configure Firewall (firewalld)

```bash
sudo firewall-cmd --zone=public --permanent --add-port=1714-1764/tcp
sudo firewall-cmd --zone=public --permanent --add-port=1714-1764/udp
sudo firewall-cmd --reload
```

## Arch Linux Installation

### Install Dependencies

```bash
sudo pacman -S --needed \
  base-devel \
  rust \
  cargo \
  pkg-config \
  libxkbcommon \
  wayland \
  wayland-protocols \
  mesa \
  libglvnd \
  pixman \
  libinput \
  libxcb \
  xcb-util \
  xcb-util-image \
  libdrm \
  fontconfig \
  freetype2 \
  systemd \
  dbus \
  libpulse \
  expat \
  openssl \
  git
```

### Install Just

```bash
cargo install just
```

### Build and Install

```bash
# Clone the repository
git clone https://github.com/olafkfreund/cosmic-applet-kdeconnect.git
cd cosmic-applet-kdeconnect

# Build and install
just build-release
sudo just install

# Enable daemon
systemctl --user enable --now kdeconnect-daemon
```

### Configure Firewall

If using `ufw`:

```bash
sudo ufw allow 1714:1764/tcp
sudo ufw allow 1714:1764/udp
```

If using `iptables`:

```bash
sudo iptables -A INPUT -p tcp --dport 1714:1764 -j ACCEPT
sudo iptables -A INPUT -p udp --dport 1714:1764 -j ACCEPT
sudo iptables-save | sudo tee /etc/iptables/iptables.rules
```

## From Source

### Prerequisites

Ensure you have:
- Rust 1.70+ with cargo
- Just command runner
- Required system libraries for your distribution (see above)

### Steps

```bash
# Clone repository
git clone https://github.com/olafkfreund/cosmic-applet-kdeconnect.git
cd cosmic-applet-kdeconnect

# Build all components
just build-release

# Option 1: Install system-wide (requires sudo)
sudo just install

# Option 2: Install to user directory (no sudo)
just install-local

# Start the daemon
systemctl --user enable --now kdeconnect-daemon
```

### Verify Installation

```bash
# Check if daemon is running
systemctl --user status kdeconnect-daemon

# Check version
cosmic-applet-kdeconnect --version
kdeconnect-daemon --version
```

## Firewall Configuration

KDE Connect requires the following ports:

- **TCP**: 1714-1764
- **UDP**: 1714-1764

### Network Discovery

The protocol uses:
- **UDP port 1716** for device discovery broadcasts
- **TCP ports 1739-1764** for data transfer connections

### Security Considerations

- Only open these ports on trusted networks (home/work WiFi)
- Do not expose these ports to the internet
- Use a firewall zone for local network only

## Mobile App Installation

### Android

1. **Google Play Store** (Official):
   - Open Play Store
   - Search for "KDE Connect"
   - Install "KDE Connect" by KDE Community

2. **F-Droid** (Open Source):
   ```
   https://f-droid.org/packages/org.kde.kdeconnect_tp/
   ```

3. **Direct APK**:
   - Download from [KDE Connect Android Releases](https://invent.kde.org/network/kdeconnect-android/-/releases)
   - Enable "Install from Unknown Sources"
   - Install the APK

### iOS

1. **App Store**:
   - Open App Store
   - Search for "KDE Connect"
   - Install "KDE Connect" by KDE e.V.

2. Direct link:
   ```
   https://apps.apple.com/app/kde-connect/id1580245991
   ```

## Post-Installation

### 1. Add Applet to Panel

1. Right-click on COSMIC panel
2. Select "Panel Settings"
3. Click "Add Applet"
4. Find "KDE Connect" in the list
5. Click to add it to your panel

### 2. Start Daemon

```bash
# Enable auto-start on login
systemctl --user enable kdeconnect-daemon

# Start immediately
systemctl --user start kdeconnect-daemon

# Check status
systemctl --user status kdeconnect-daemon
```

### 3. Configure Permissions

The daemon needs:
- Network access (automatically granted)
- File system access for downloads (check `~/.local/share/kdeconnect/`)
- Notification permissions (COSMIC handles this)

### 4. First Pairing

See [USER_GUIDE.md](USER_GUIDE.md) for detailed pairing instructions.

## Troubleshooting

If you encounter issues, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common problems and solutions.

### Quick Checks

```bash
# Check if daemon is running
systemctl --user status kdeconnect-daemon

# View daemon logs
journalctl --user -u kdeconnect-daemon -f

# Test network connectivity
ping -c 3 <your-device-ip>

# Check firewall rules
sudo iptables -L -n | grep 1716
```

## Uninstallation

### System-wide Installation

```bash
sudo rm /usr/local/bin/cosmic-applet-kdeconnect
sudo rm /usr/local/bin/kdeconnect-daemon
systemctl --user disable --now kdeconnect-daemon
rm ~/.config/systemd/user/kdeconnect-daemon.service
```

### User Installation

```bash
rm ~/.local/bin/cosmic-applet-kdeconnect
rm ~/.local/bin/kdeconnect-daemon
systemctl --user disable --now kdeconnect-daemon
```

### Remove Configuration

```bash
rm -rf ~/.config/kdeconnect
rm -rf ~/.local/share/kdeconnect
```

## Updating

### From Source

```bash
cd cosmic-applet-kdeconnect
git pull origin main
just build-release
sudo just install
systemctl --user restart kdeconnect-daemon
```

### NixOS

```bash
nix flake update
sudo nixos-rebuild switch
```

## Support

- **Documentation**: [docs/](https://github.com/olafkfreund/cosmic-applet-kdeconnect/tree/main/docs)
- **Issues**: [GitHub Issues](https://github.com/olafkfreund/cosmic-applet-kdeconnect/issues)
- **Community**: [COSMIC Desktop Chat](https://chat.pop-os.org/)
