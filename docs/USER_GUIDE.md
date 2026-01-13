# User Guide

Complete guide to using COSMIC KDE Connect for device synchronization and file sharing.

## Table of Contents

- [Getting Started](#getting-started)
- [First Time Setup](#first-time-setup)
- [Pairing Devices](#pairing-devices)
- [Using Features](#using-features)
- [Managing Devices](#managing-devices)
- [Plugin Guide](#plugin-guide)
- [Tips and Tricks](#tips-and-tricks)

## Getting Started

### Prerequisites

Before using COSMIC KDE Connect, ensure:

1. **Desktop Requirements**
   - COSMIC Desktop Environment installed
   - KDE Connect applet installed (see [INSTALL.md](INSTALL.md))
   - Daemon running: `systemctl --user status kdeconnect-daemon`

2. **Mobile Requirements**
   - KDE Connect app installed on your phone/tablet
   - Both devices connected to the same WiFi network
   - Firewall ports 1714-1764 open (TCP and UDP)

3. **Network Requirements**
   - Both devices on the same local network
   - No VPN blocking local traffic
   - Router allows device-to-device communication

## First Time Setup

### Step 1: Add Applet to Panel

1. Right-click on your COSMIC panel (top bar)
2. Select **"Panel Settings"** or **"Configure Panel"**
3. Click **"Add Applet"** or **"Add Widget"**
4. Scroll to find **"KDE Connect"** in the list
5. Click to add it to your panel
6. Close the panel settings

You should now see the KDE Connect icon (phone symbol) in your panel.

### Step 2: Start the Daemon

The daemon should start automatically on login, but if not:

```bash
# Enable auto-start
systemctl --user enable kdeconnect-daemon

# Start now
systemctl --user start kdeconnect-daemon

# Verify it's running
systemctl --user status kdeconnect-daemon
```

Expected output:
```
● kdeconnect-daemon.service - KDE Connect Daemon
   Loaded: loaded
   Active: active (running)
```

### Step 3: Install Mobile App

#### Android

**Option 1: Google Play Store**
1. Open Play Store
2. Search "KDE Connect"
3. Install "KDE Connect" by KDE Community

**Option 2: F-Droid** (Open Source)
1. Open F-Droid app
2. Search "KDE Connect"
3. Install

#### iOS

1. Open App Store
2. Search "KDE Connect"
3. Install "KDE Connect" by KDE e.V.

## Pairing Devices

### Automatic Discovery

When both devices are on the same network, they should discover each other automatically.

### Pairing Process

#### From Desktop (COSMIC)

1. **Open the Applet**
   - Click the KDE Connect icon in your panel
   - A popup will show available devices

2. **Find Your Device**
   - Look for your phone/tablet name in the list
   - You should see: `[Device Name] - Not paired`

3. **Initiate Pairing**
   - Click the **"Pair"** button next to your device
   - Or click the device name, then click **"Request Pairing"**

4. **Accept on Mobile**
   - A notification will appear on your phone/tablet
   - Tap the notification
   - Tap **"Accept"** in the KDE Connect app

5. **Confirmation**
   - Desktop will show: `[Device Name] - Paired ✓`
   - Green checkmark indicates successful pairing

#### From Mobile Device

1. **Open KDE Connect App**
2. **Find Your Desktop**
   - You should see your computer in the "Available devices" list
   - Example: "MyDesktop - COSMIC"

3. **Tap to Pair**
   - Tap on your computer name
   - Tap **"Request pairing"**

4. **Accept on Desktop**
   - Click the KDE Connect panel icon
   - Click **"Accept"** for the pairing request

### Troubleshooting Discovery

If devices don't appear:

1. **Check Network**
   ```bash
   # Verify devices can reach each other
   ping [your-phone-ip]
   ```

2. **Check Firewall**
   ```bash
   # Verify ports are open
   sudo iptables -L -n | grep 1716
   ```

3. **Restart Services**
   ```bash
   # On desktop
   systemctl --user restart kdeconnect-daemon

   # On mobile: Force close and reopen KDE Connect app
   ```

4. **Manual Connection** (Advanced)
   - Get device IP address
   - In KDE Connect settings, add device by IP

## Using Features

### File Sharing

#### Send File from Desktop to Mobile

1. **Method 1: Via Applet**
   - Click KDE Connect icon
   - Click your device name
   - Click **"Send File"**
   - Browse and select file
   - Click **"Open"**

2. **Method 2: Via File Manager** (Future)
   - Right-click file in COSMIC Files
   - Select **"Send via KDE Connect"**
   - Choose destination device

3. **Method 3: Via Command Line**
   ```bash
   kdeconnect-cli -d [device-id] --share /path/to/file
   ```

#### Send File from Mobile to Desktop

1. Open any app with share functionality
2. Tap the **Share** button
3. Select **"KDE Connect"**
4. Choose your desktop
5. File appears in: `~/Downloads/` or `~/.local/share/kdeconnect/`

#### Send URL/Text

**From Desktop:**
```bash
kdeconnect-cli -d [device-id] --share-text "Hello from desktop!"
```

**From Mobile:**
1. Select text in any app
2. Tap **Share**
3. Choose **KDE Connect**
4. Select your desktop

### Clipboard Synchronization

Clipboard sync keeps your clipboard synchronized between devices.

#### Enable Clipboard Plugin

1. Click KDE Connect icon
2. Click your device
3. Click **"Plugin Settings"** or gear icon
4. Enable **"Clipboard"** plugin

#### Using Clipboard Sync

Once enabled, clipboard is automatically synchronized:

1. **Copy on Phone → Paste on Desktop**
   - Copy text on your phone
   - Immediately paste (Ctrl+V) on desktop

2. **Copy on Desktop → Paste on Phone**
   - Copy text on desktop (Ctrl+C)
   - Long-press paste on phone

**Security Note**: Clipboard contains sensitive data. Only pair trusted devices.

### Battery Monitoring

View your phone's battery level from your desktop.

#### Enable Battery Plugin

1. Click KDE Connect icon
2. Select your device
3. Enable **"Battery"** plugin

#### View Battery Status

Battery level appears next to device name in the applet:
```
My Phone - Paired ✓
Battery: 85% (Charging)
```

#### Low Battery Notifications

Receive notifications when your phone's battery is low:
- Appears as COSMIC notification
- Default threshold: 15%
- Can be customized in settings

### Notification Mirroring

Receive your phone's notifications on your desktop.

#### Enable Notifications Plugin

**On Mobile:**
1. Open KDE Connect app
2. Tap your desktop device
3. Enable **"Notification sync"**
4. Grant notification access permission

**On Desktop:**
1. Click KDE Connect icon
2. Select device
3. Enable **"Notification"** plugin

#### Using Notifications

Phone notifications appear as COSMIC notifications on your desktop:
- Same title and content
- Click to open on phone (if supported)
- Dismiss on one device → dismisses on all

#### Filter Notifications

Configure which apps send notifications:

**On Mobile:**
1. KDE Connect app → Device → Plugins
2. Tap **"Notification sync"**
3. **"Applications"** → Select which apps to sync

### Media Control (MPRIS)

Control desktop media players from your phone.

#### Enable MPRIS Plugin

**On Desktop:**
1. Click KDE Connect icon
2. Select device
3. Enable **"MPRIS"** plugin

**On Mobile:**
1. Open KDE Connect app
2. Tap desktop device
3. Enable **"Multimedia control receiver"**

#### Controlling Media

**From Mobile:**
1. Open KDE Connect app
2. Tap **"Multimedia control"**
3. See currently playing media
4. Use play/pause/skip controls

**Supported Players:**
- Firefox/Chrome browser audio
- VLC Media Player
- Spotify
- Any MPRIS2-compatible player

### Ping

Test connection between devices.

#### Send Ping

**From Desktop:**
1. Click KDE Connect icon
2. Select device
3. Click **"Ping"** button

**From Mobile:**
1. Open KDE Connect app
2. Tap device
3. Tap **"Ping"** button

**Result**: Other device shows notification with ping message.

### Remote Input (Future Feature)

Use your phone as a touchpad and keyboard.

#### Enable Remote Input

**On Desktop:**
1. Enable **"Remote input"** plugin
2. Grant input permissions if prompted

**On Mobile:**
1. Open KDE Connect app
2. Tap **"Remote input"**

#### Using Remote Input

- **Touchpad Mode**: Swipe to move mouse cursor
- **Keyboard Mode**: Tap keyboard icon, type on phone
- **Click**: Single tap
- **Right Click**: Two-finger tap
- **Scroll**: Two-finger swipe

### Run Commands (Future Feature)

Execute predefined commands on your desktop from your phone.

#### Setup Commands

**On Desktop:**
1. Click KDE Connect icon
2. Select device → Settings
3. Go to **"Run command"** plugin
4. Add new command:
   - Name: Lock Screen
   - Command: `loginctl lock-session`

**On Mobile:**
1. Open KDE Connect app
2. Tap **"Run command"**
3. See available commands
4. Tap to execute

#### Example Commands

```bash
# Lock screen
loginctl lock-session

# Take screenshot
gnome-screenshot -f ~/screenshot.png

# Suspend system
systemctl suspend

# Check disk space
df -h
```

## Managing Devices

### Device Information

View device details:

1. Click KDE Connect icon
2. Click device name
3. View device information:
   - Device name and type
   - Battery level
   - Connection status
   - IP address
   - Last seen timestamp

### Rename Device

**On Desktop:**
1. Click KDE Connect icon
2. Click device → Settings
3. Change **"Device name"**

**On Mobile:**
1. KDE Connect app → Settings
2. Change **"Device name"**

### Unpair Device

**From Desktop:**
1. Click KDE Connect icon
2. Click device
3. Click **"Unpair"** button
4. Confirm action

**From Mobile:**
1. Open KDE Connect app
2. Tap device
3. Tap **"Unpair"**

### Reconnect Device

If device shows as disconnected:

1. **Automatic Reconnection**
   - Usually happens automatically
   - Wait 10-30 seconds

2. **Manual Reconnection**
   - Click **"Refresh"** in applet
   - Or restart daemon: `systemctl --user restart kdeconnect-daemon`

3. **Force Rediscovery**
   ```bash
   # Stop daemon
   systemctl --user stop kdeconnect-daemon

   # Clear cache
   rm -rf ~/.config/kdeconnect/*

   # Start daemon
   systemctl --user start kdeconnect-daemon

   # Re-pair devices
   ```

### Multiple Devices

You can pair multiple devices simultaneously:

- Each device appears separately in the applet
- Configure plugins independently per device
- All devices can share files with desktop

**Example Setup:**
- Phone: Clipboard + Notifications + Battery
- Tablet: File sharing + Media control
- Work Phone: Ping only (security)

## Plugin Guide

### Available Plugins

| Plugin | Desktop → Mobile | Mobile → Desktop | Description |
|--------|-----------------|------------------|-------------|
| **Ping** | ✅ | ✅ | Test connectivity |
| **Battery** | ✅ | ❌ | Monitor phone battery |
| **Notification** | ❌ | ✅ | Mirror notifications |
| **Share** | ✅ | ✅ | Share files and URLs |
| **Clipboard** | ✅ | ✅ | Sync clipboard |
| **MPRIS** | ✅ | ✅ | Control media playback |
| **Remote Input** | 🚧 | 🚧 | Use phone as input device |
| **Run Command** | 🚧 | ❌ | Execute desktop commands |
| **SMS** | 🚧 | 🚧 | Send/receive SMS |
| **Find Device** | 🚧 | 🚧 | Make device ring |

✅ Available | 🚧 Coming Soon | ❌ Not Applicable

### Plugin Permissions

Some plugins require additional permissions:

**Notification Sync:**
- Android: Notification Access
- iOS: Notification permissions

**Clipboard:**
- Desktop: Clipboard access (automatic)
- Mobile: Background access

**Remote Input:**
- Desktop: Input device permissions
- Mobile: Accessibility service (Android)

**Battery:**
- Mobile: Battery stats access (automatic)

## Tips and Tricks

### Performance Tips

1. **Disable Unused Plugins**
   - Each plugin uses resources
   - Disable plugins you don't use
   - Improves battery life on mobile

2. **Adjust Sync Frequency**
   - Some plugins sync continuously
   - Adjust in plugin settings
   - Balance between performance and features

3. **Use WiFi, Not Mobile Data**
   - KDE Connect works on same network only
   - More secure
   - Better performance

### Security Best Practices

1. **Only Pair Trusted Devices**
   - Pairing grants significant access
   - Review paired devices regularly
   - Unpair old/unused devices

2. **Use Trusted Networks**
   - Home or work WiFi
   - Avoid public WiFi for pairing
   - VPN may interfere

3. **Review Plugin Permissions**
   - Enable only needed plugins
   - Review what each plugin accesses
   - Disable sensitive plugins when not needed

4. **Certificate Verification**
   - First pairing uses TLS certificates
   - Verify device identity during pairing
   - Certificates stored in `~/.config/kdeconnect/`

### Workflow Examples

#### Developer Workflow

1. **Code on Desktop**
2. **Copy error message** (Ctrl+C)
3. **Paste in phone** to Google it on the go
4. **Find solution**
5. **Copy fix** on phone
6. **Paste in desktop IDE**

#### Media Workflow

1. **Start movie** on VLC (desktop)
2. **Control playback** from phone
3. **Pause/play** without getting up
4. **Adjust volume** from another room

#### File Sharing Workflow

1. **Take photo** on phone
2. **Share via KDE Connect** to desktop
3. **Photo appears** in Downloads
4. **Edit in GIMP** immediately

#### Notification Workflow

1. **Get WhatsApp message** on phone
2. **See notification** on desktop
3. **Continue working** without picking up phone
4. **Reply during break**

## Keyboard Shortcuts

| Action | Shortcut | Description |
|--------|----------|-------------|
| Open Applet | *Panel Click* | Show device list |
| Send File | *Drag & Drop* | Drop file on applet |
| Quick Ping | *Double Click Device* | Send ping to device |
| Refresh | *F5 in Applet* | Refresh device list |

## Command Line Interface

For automation and scripting:

### List Devices
```bash
kdeconnect-cli -l
```

### Send File
```bash
kdeconnect-cli -d [device-id] --share /path/to/file
```

### Send Text
```bash
kdeconnect-cli -d [device-id] --share-text "Hello!"
```

### Ping Device
```bash
kdeconnect-cli -d [device-id] --ping
```

### Device Info
```bash
kdeconnect-cli -d [device-id] --info
```

### Ring Device
```bash
kdeconnect-cli -d [device-id] --ring
```

## Troubleshooting

For detailed troubleshooting, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md).

### Quick Fixes

**Devices Not Appearing:**
```bash
systemctl --user restart kdeconnect-daemon
```

**Connection Keeps Dropping:**
- Check WiFi stability
- Verify firewall rules
- Check router settings

**File Transfer Fails:**
- Check disk space
- Verify file permissions
- Try smaller files first

**Notifications Not Syncing:**
- Verify plugin enabled on both devices
- Check notification permissions on mobile
- Restart KDE Connect app

## Getting Help

- **Documentation**: See [docs/](https://github.com/olafkfreund/cosmic-applet-kdeconnect/tree/main/docs)
- **Issues**: [GitHub Issues](https://github.com/olafkfreund/cosmic-applet-kdeconnect/issues)
- **Community**: [COSMIC Desktop Chat](https://chat.pop-os.org/)
- **KDE Connect**: [Official Documentation](https://kdeconnect.kde.org/)

## Advanced Configuration

### Configuration Files

**Desktop Configuration:**
```
~/.config/kdeconnect/
├── config                    # Main configuration
├── [device-id]/              # Per-device settings
│   ├── certificate.pem       # Device certificate
│   └── plugins/              # Plugin configurations
└── trusted_devices.json      # Paired devices
```

**Storage Locations:**
```
~/.local/share/kdeconnect/    # Received files
~/.local/share/kdeconnect/downloads/  # Downloads
```

### Custom Storage Path

To change where received files are stored:

1. Edit: `~/.config/kdeconnect/config`
2. Add:
   ```ini
   [General]
   downloadPath=/custom/path/
   ```
3. Restart daemon

### Firewall Configuration

**Detailed Firewall Rules:**

```bash
# For firewalld (Fedora/RHEL)
sudo firewall-cmd --permanent --add-port=1714-1764/tcp
sudo firewall-cmd --permanent --add-port=1714-1764/udp
sudo firewall-cmd --reload

# For ufw (Ubuntu/Debian)
sudo ufw allow 1714:1764/tcp
sudo ufw allow 1714:1764/udp

# For iptables
sudo iptables -A INPUT -p tcp --dport 1714:1764 -j ACCEPT
sudo iptables -A INPUT -p udp --dport 1714:1764 -j ACCEPT
```

**NixOS Configuration:**

Add to your `configuration.nix`:
```nix
networking.firewall = {
  allowedTCPPortRanges = [
    { from = 1714; to = 1764; }
  ];
  allowedUDPPortRanges = [
    { from = 1714; to = 1764; }
  ];
};
```

## FAQ

**Q: Can I use KDE Connect without COSMIC Desktop?**
A: This applet is specifically for COSMIC, but KDE Connect works on any Linux desktop. Use the official KDE Connect desktop app.

**Q: Does this work over the internet?**
A: No, devices must be on the same local network. This is a security feature.

**Q: Can I pair multiple phones to one desktop?**
A: Yes, you can pair unlimited devices.

**Q: Is my data encrypted?**
A: Yes, all communication uses TLS encryption with self-signed certificates.

**Q: Does this drain my phone battery?**
A: Minimal impact when idle. Plugins like notification sync use more battery.

**Q: Can I use this on mobile data?**
A: No, requires local network connection (WiFi).

**Q: What's the file size limit for transfers?**
A: No hard limit, but large files (>2GB) may be slow. Use USB for huge files.

**Q: Can I control my desktop from my phone?**
A: Remote input plugin allows mouse/keyboard control (coming soon).

**Q: Is this compatible with official KDE Connect?**
A: Yes, fully compatible with KDE Connect protocol v7.

---

**Last Updated**: 2026-01-13
**Version**: 1.0.0
**Status**: Active Development

For installation help, see [INSTALL.md](INSTALL.md)
For troubleshooting, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
