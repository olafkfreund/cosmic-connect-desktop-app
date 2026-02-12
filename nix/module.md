# COSMIC Connect NixOS Module {#cosmic-connect-nixos-module}

This NixOS module provides declarative configuration for COSMIC Connect, enabling device connectivity features for COSMIC Desktop.

## Quick Start {#quick-start}

Enable the service in your NixOS configuration:

```nix
services.cosmic-connect.enable = true;
```

## Configuration Options {#configuration-options}

### Basic Configuration {#basic-configuration}

#### `services.cosmic-connect.enable` {#services-cosmic-connect-enable}

**Type:** `boolean`
**Default:** `false`

Enable COSMIC Connect device connectivity service.

#### `services.cosmic-connect.package` {#services-cosmic-connect-package}

**Type:** `package`
**Default:** `pkgs.cosmic-connect`

The COSMIC Connect package to use. Override this to use a custom build or different version.

**Example:**
```nix
services.cosmic-connect.package = pkgs.cosmic-connect.override {
  enableFeature = true;
};
```

#### `services.cosmic-connect.openFirewall` {#services-cosmic-connect-openfirewall}

**Type:** `boolean`
**Default:** `true`

Whether to automatically open firewall ports for COSMIC Connect.

- TCP/UDP 1814-1864: Device discovery (CConnect protocol)
- TCP 1739-1764: File transfer

**Example:**
```nix
services.cosmic-connect.openFirewall = false;  # Manage firewall manually
```

---

### Daemon Configuration {#daemon-configuration}

#### `services.cosmic-connect.daemon.enable` {#services-cosmic-ext-connect-daemon-enable}

**Type:** `boolean`
**Default:** `true`

Enable the COSMIC Connect daemon as a user service. The daemon handles:
- Device discovery and pairing
- Plugin communication
- Background tasks

#### `services.cosmic-connect.daemon.autoStart` {#services-cosmic-ext-connect-daemon-autostart}

**Type:** `boolean`
**Default:** `true`

Automatically start the daemon on user login.

#### `services.cosmic-connect.daemon.logLevel` {#services-cosmic-ext-connect-daemon-loglevel}

**Type:** `enum [ "error" "warn" "info" "debug" "trace" ]`
**Default:** `"info"`

Set the logging verbosity level.

**Example:**
```nix
services.cosmic-connect.daemon.logLevel = "debug";  # For troubleshooting
```

#### `services.cosmic-connect.daemon.settings` {#services-cosmic-ext-connect-daemon-settings}

**Type:** `attrsOf anything`
**Default:** `{}`

Additional daemon configuration settings. These are merged with plugin configuration and written to `/etc/xdg/cosmic-connect/daemon.toml`.

**Example:**
```nix
services.cosmic-connect.daemon.settings = {
  discovery = {
    broadcast_interval = 5000;  # milliseconds
    listen_port = 1816;
  };
  security = {
    certificate_dir = "~/.config/cosmic-connect/certs";
  };
};
```

---

### Applet Configuration {#applet-configuration}

#### `services.cosmic-connect.applet.enable` {#services-cosmic-connect-applet-enable}

**Type:** `boolean`
**Default:** `true`

Enable the COSMIC panel applet for quick access to connected devices and features.

---

### Plugin Configuration {#plugin-configuration}

All plugins default to `true` unless noted. Plugins can be enabled/disabled globally here, and per-device in the UI.

#### Core Plugins {#core-plugins}

##### `services.cosmic-connect.plugins.battery` {#services-cosmic-connect-plugins-battery}

**Default:** `true`

Monitor battery status from mobile devices.

##### `services.cosmic-connect.plugins.clipboard` {#services-cosmic-connect-plugins-clipboard}

**Default:** `true`

Synchronize clipboard content between devices.

##### `services.cosmic-connect.plugins.notification` {#services-cosmic-connect-plugins-notification}

**Default:** `true`

Mirror notifications from mobile devices to desktop.

##### `services.cosmic-connect.plugins.share` {#services-cosmic-connect-plugins-share}

**Default:** `true`

Share files and URLs between devices.

##### `services.cosmic-connect.plugins.mpris` {#services-cosmic-connect-plugins-mpris}

**Default:** `true`

Control media players via MPRIS protocol.

##### `services.cosmic-connect.plugins.ping` {#services-cosmic-connect-plugins-ping}

**Default:** `true`

Test connectivity between devices.

#### Advanced Plugins {#advanced-plugins}

##### `services.cosmic-connect.plugins.remotedesktop` {#services-cosmic-connect-plugins-remotedesktop}

**Default:** `false` (security-sensitive)

Enable VNC-based remote desktop for screen sharing and remote control.

**Requirements:**
- PipeWire
- Wayland portal support
- Explicit opt-in required

**Example:**
```nix
services.cosmic-connect.plugins.remotedesktop = true;
```

##### `services.cosmic-connect.plugins.runcommand` {#services-cosmic-connect-plugins-runcommand}

**Default:** `false` (security-sensitive)

Execute predefined commands on paired devices.

**Security:** Disabled by default. Only enable if you trust all paired devices.

##### `services.cosmic-connect.plugins.remoteinput` {#services-cosmic-connect-plugins-remoteinput}

**Default:** `true`

Remote mouse and keyboard control. Useful for presentations and remote assistance.

##### `services.cosmic-connect.plugins.findmyphone` {#services-cosmic-connect-plugins-findmyphone}

**Default:** `true`

Trigger audio alerts on devices to help locate them.

##### `services.cosmic-connect.plugins.lock` {#services-cosmic-connect-plugins-lock}

**Default:** `true`

Lock and unlock desktop sessions remotely.

##### `services.cosmic-connect.plugins.telephony` {#services-cosmic-connect-plugins-telephony}

**Default:** `true`

Display SMS and call notifications from mobile devices.

##### `services.cosmic-connect.plugins.presenter` {#services-cosmic-connect-plugins-presenter}

**Default:** `false` (specialized use)

Laser pointer and presentation controls for slideshow applications.

##### `services.cosmic-connect.plugins.contacts` {#services-cosmic-connect-plugins-contacts}

**Default:** `false`

Synchronize contact lists between devices via vCard format.

##### `services.cosmic-connect.plugins.systemmonitor` {#services-cosmic-connect-plugins-systemmonitor}

**Default:** `true`

Share desktop resource monitoring (CPU, memory, disk, network).

##### `services.cosmic-connect.plugins.wol` {#services-cosmic-connect-plugins-wol}

**Default:** `true`

Send Wake-on-LAN magic packets to wake sleeping desktops.

##### `services.cosmic-connect.plugins.screenshot` {#services-cosmic-connect-plugins-screenshot}

**Default:** `true`

Capture and transfer screenshots from remote desktops.

---

### Security Configuration {#security-configuration}

#### `services.cosmic-connect.security.certificateDirectory` {#services-cosmic-connect-security-certificatedirectory}

**Type:** `string`
**Default:** `"~/.config/cosmic-connect/certs"`

Directory where device certificates are stored for TLS communication.

#### `services.cosmic-connect.security.trustOnFirstPair` {#services-cosmic-connect-security-trustonfirstpair}

**Type:** `boolean`
**Default:** `true`

Trust devices automatically on first pairing without manual verification.

**Security Recommendation:** Disable in untrusted network environments.

**Example:**
```nix
services.cosmic-connect.security.trustOnFirstPair = false;  # Enhanced security
```

---

### Storage Configuration {#storage-configuration}

#### `services.cosmic-connect.storage.downloadDirectory` {#services-cosmic-connect-storage-downloaddirectory}

**Type:** `string`
**Default:** `"~/Downloads"`

Directory where received files are saved.

#### `services.cosmic-connect.storage.dataDirectory` {#services-cosmic-connect-storage-datadirectory}

**Type:** `string`
**Default:** `"~/.local/share/cosmic-connect"`

Base directory for COSMIC Connect application data.

---

## Example Configurations {#example-configurations}

### Minimal Configuration {#minimal-configuration}

```nix
services.cosmic-connect.enable = true;
```

### Security-Hardened Configuration {#security-hardened-configuration}

```nix
services.cosmic-connect = {
  enable = true;
  openFirewall = true;

  daemon = {
    enable = true;
    logLevel = "info";
  };

  plugins = {
    # Disable security-sensitive plugins
    remotedesktop = false;
    runcommand = false;
    remoteinput = false;

    # Enable only essential plugins
    battery = true;
    notification = true;
    share = true;
  };

  security = {
    trustOnFirstPair = false;  # Require manual verification
  };
};
```

### Developer Configuration {#developer-configuration}

```nix
services.cosmic-connect = {
  enable = true;

  daemon = {
    enable = true;
    autoStart = true;
    logLevel = "debug";  # Verbose logging
  };

  plugins = {
    # Enable all plugins for testing
    battery = true;
    clipboard = true;
    notification = true;
    share = true;
    mpris = true;
    ping = true;
    remotedesktop = true;  # Include experimental features
    runcommand = true;
    remoteinput = true;
    findmyphone = true;
    lock = true;
    telephony = true;
    presenter = true;
    contacts = true;
    systemmonitor = true;
    wol = true;
    screenshot = true;
  };
};
```

### Presentation Mode {#presentation-mode}

```nix
services.cosmic-connect = {
  enable = true;

  plugins = {
    presenter = true;      # Laser pointer controls
    remoteinput = true;    # Remote clicker
    notification = false;  # Disable distractions
  };
};
```

---

## Firewall Configuration {#firewall-configuration}

COSMIC Connect requires specific firewall ports for device discovery and communication.

### Automatic (Recommended) {#automatic-recommended}

```nix
services.cosmic-connect.openFirewall = true;  # Default
```

### Manual Firewall Rules {#manual-firewall-rules}

```nix
services.cosmic-connect.openFirewall = false;

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

---

## Systemd Service {#systemd-service}

The daemon runs as a user systemd service with security hardening:

- **Service name:** `cosmic-ext-connect-daemon.service`
- **Type:** User service (per-user instance)
- **Restart policy:** On failure, with 5s delay
- **Security:** NoNewPrivileges, ProtectSystem, PrivateTmp, and more

### Service Management {#service-management}

```bash
# Start daemon {#start-daemon}
systemctl --user start cosmic-ext-connect-daemon

# Stop daemon {#stop-daemon}
systemctl --user stop cosmic-ext-connect-daemon

# Check status {#check-status}
systemctl --user status cosmic-ext-connect-daemon

# View logs {#view-logs}
journalctl --user -u cosmic-ext-connect-daemon -f
```

---

## Troubleshooting {#troubleshooting}

### Devices Not Discovering {#devices-not-discovering}

1. **Check firewall:**
   ```nix
   services.cosmic-connect.openFirewall = true;
   ```

2. **Verify daemon is running:**
   ```bash
   systemctl --user status cosmic-ext-connect-daemon
   ```

3. **Enable debug logging:**
   ```nix
   services.cosmic-connect.daemon.logLevel = "debug";
   ```

4. **Check network connectivity:**
   ```bash
   ss -tulpn | grep 1816
   ```

### Permission Issues {#permission-issues}

The daemon requires access to specific directories:
- `~/.config/cosmic/cosmic-connect` (config)
- `~/.local/share/cosmic/cosmic-connect` (data)

These are configured in the systemd service's `ReadWritePaths`.

### Plugin Issues {#plugin-issues}

Check if a plugin is enabled:
```nix
services.cosmic-connect.plugins.<plugin-name> = true;
```

Plugins can also be disabled per-device via the applet UI.

---

## Integration with COSMIC Desktop {#integration-with-cosmic-desktop}

COSMIC Connect integrates seamlessly with COSMIC Desktop:

- **Panel Applet:** Shows connected devices and battery status
- **Notifications:** Uses COSMIC notification system
- **File Picker:** Integrates with COSMIC file manager
- **Settings:** Uses COSMIC theming and design patterns

---

## Compatibility {#compatibility}

### Supported Platforms {#supported-platforms}

- **Operating System:** NixOS (Linux)
- **Architecture:** x86_64-linux, aarch64-linux
- **Desktop Environment:** COSMIC Desktop (recommended)

### Compatible Devices {#compatible-devices}

- Android devices with KDE Connect app
- Other Linux desktops with KDE Connect
- COSMIC Desktop machines with COSMIC Connect

### Protocol {#protocol}

COSMIC Connect implements the CConnect protocol (port 1816), which is side-by-side compatible with KDE Connect protocol (port 1714-1764).

---

## Security Considerations {#security-considerations}

### Network Security {#network-security}

- All device communication is encrypted via TLS
- Each paired device has a unique certificate
- Certificates are stored in `~/.config/cosmic-connect/certs`

### Systemd Hardening {#systemd-hardening}

The daemon runs with extensive systemd security hardening:
- No new privileges (`NoNewPrivileges=true`)
- Protected system directories (`ProtectSystem=strict`)
- Private temporary files (`PrivateTmp=true`)
- Memory restrictions (`MemoryMax=1G`)
- Task limits (`TasksMax=1000`)

### Plugin Security {#plugin-security}

Some plugins are disabled by default due to security implications:
- `remotedesktop`: Allows screen sharing and remote control
- `runcommand`: Allows remote command execution
- `presenter`: Specialized use case

Always review plugin permissions before enabling them.

---

## Additional Resources {#additional-resources}

- **GitHub:** https://github.com/olafkfreund/cosmic-connect-desktop-app
- **Documentation:** See `README.md` in the repository
- **Bug Reports:** https://github.com/olafkfreund/cosmic-connect-desktop-app/issues
- **KDE Connect Protocol:** https://community.kde.org/KDEConnect

---

## Maintainers {#maintainers}

Add your name when contributing:
```nix
meta.maintainers = with maintainers; [ your-github-username ];
```
