# Troubleshooting Guide

Solutions to common problems with COSMIC KDE Connect.

## Table of Contents

- [Connection Issues](#connection-issues)
- [Pairing Problems](#pairing-problems)
- [File Transfer Issues](#file-transfer-issues)
- [Plugin Problems](#plugin-problems)
- [Performance Issues](#performance-issues)
- [System-Specific Issues](#system-specific-issues)
- [Error Messages](#error-messages)
- [Debug Mode](#debug-mode)
- [Getting Help](#getting-help)

## Connection Issues

### Devices Not Appearing

**Problem**: Desktop and mobile device don't see each other.

**Solutions**:

1. **Verify Same Network**
   ```bash
   # Get desktop IP
   ip addr show | grep inet

   # On mobile: Settings → WiFi → Check connected network
   # Both must be on same WiFi network
   ```

2. **Check Firewall**
   ```bash
   # Test if ports are open
   sudo netstat -tuln | grep 1716

   # If nothing shows, open ports:
   sudo ufw allow 1714:1764/tcp
   sudo ufw allow 1714:1764/udp
   ```

3. **Restart Services**
   ```bash
   # Restart daemon
   systemctl --user restart kdeconnect-daemon

   # Check status
   systemctl --user status kdeconnect-daemon
   ```

4. **Check Daemon Logs**
   ```bash
   journalctl --user -u kdeconnect-daemon -f
   ```

5. **Router Configuration**
   - Some routers block device-to-device communication
   - Check for "AP Isolation" or "Client Isolation" setting
   - Disable it in router settings
   - Common on guest networks

### Device Shows as Offline

**Problem**: Previously paired device shows as offline or unreachable.

**Solutions**:

1. **Check Device Status**
   ```bash
   kdeconnect-cli -l
   ```

   Look for `(reachable)` or `(paired and reachable)` status.

2. **Verify IP Address**
   ```bash
   # Get current IP
   kdeconnect-cli -d [device-id] --info

   # Ping the device
   ping [device-ip]
   ```

3. **Force Refresh**
   ```bash
   systemctl --user restart kdeconnect-daemon
   ```

4. **Check Mobile App**
   - Is KDE Connect app running?
   - Check battery optimization settings
   - Disable battery optimization for KDE Connect

### Connection Keeps Dropping

**Problem**: Device connects then disconnects repeatedly.

**Solutions**:

1. **Check WiFi Stability**
   ```bash
   # Monitor connection quality
   watch -n 1 'ping -c 1 [device-ip] | grep time'
   ```

2. **Mobile Battery Optimization**
   - Android: Settings → Apps → KDE Connect → Battery → Unrestricted
   - iOS: Background App Refresh enabled

3. **Router Power Save**
   - Some routers have power-saving features
   - Can drop idle connections
   - Check router settings

4. **Network Load**
   - High network traffic can affect discovery
   - Try during off-peak hours
   - Consider static IP addresses

### VPN Interference

**Problem**: Can't connect while VPN is active.

**Solutions**:

1. **Split Tunneling**
   - Configure VPN to exclude local network
   - Add `192.168.0.0/16` to VPN exceptions
   - Varies by VPN client

2. **Disable VPN Temporarily**
   - Disconnect VPN
   - Pair and test connection
   - Re-enable VPN after setup

3. **Use VPN on Mobile Only**
   - Keep desktop without VPN
   - Or vice versa
   - One device must be on local network

## Pairing Problems

### Pairing Request Not Appearing

**Problem**: Sent pairing request but other device doesn't show notification.

**Solutions**:

1. **Check Notification Permissions**
   - Android: Settings → Apps → KDE Connect → Notifications → Enabled
   - iOS: Settings → KDE Connect → Notifications → Allow

2. **Manual Pairing Acceptance**
   ```bash
   # List pending pairing requests
   kdeconnect-cli -l

   # Accept pairing
   kdeconnect-cli -d [device-id] --pair
   ```

3. **Clear Old Pairing Data**
   ```bash
   # Stop daemon
   systemctl --user stop kdeconnect-daemon

   # Remove old certificates
   rm -rf ~/.config/kdeconnect/[device-id]/

   # Start daemon
   systemctl --user start kdeconnect-daemon
   ```

### Pairing Failed - Certificate Error

**Problem**: "Certificate verification failed" or similar error.

**Solutions**:

1. **Regenerate Certificates**
   ```bash
   # Stop daemon
   systemctl --user stop kdeconnect-daemon

   # Remove certificates
   rm -rf ~/.config/kdeconnect/*/certificate.pem
   rm -rf ~/.config/kdeconnect/*/privateKey.pem

   # Start daemon (generates new certificates)
   systemctl --user start kdeconnect-daemon
   ```

2. **Check System Time**
   ```bash
   # Certificates are time-sensitive
   date

   # Sync time if wrong
   sudo timedatectl set-ntp true
   ```

3. **Complete Reset**
   ```bash
   # Nuclear option - remove all KDE Connect data
   systemctl --user stop kdeconnect-daemon
   rm -rf ~/.config/kdeconnect/
   rm -rf ~/.local/share/kdeconnect/
   systemctl --user start kdeconnect-daemon
   ```

### Can't Unpair Device

**Problem**: Unpair button doesn't work or device reappears.

**Solutions**:

1. **Force Unpair**
   ```bash
   kdeconnect-cli -d [device-id] --unpair
   ```

2. **Manual Removal**
   ```bash
   systemctl --user stop kdeconnect-daemon
   rm -rf ~/.config/kdeconnect/[device-id]/
   systemctl --user start kdeconnect-daemon
   ```

3. **Unpair from Both Devices**
   - Unpair from desktop
   - Unpair from mobile app
   - Restart both

## File Transfer Issues

### File Transfer Fails

**Problem**: Files won't send or receive.

**Solutions**:

1. **Check Disk Space**
   ```bash
   df -h ~
   ```

2. **Check Permissions**
   ```bash
   # Verify write access to download folder
   ls -la ~/.local/share/kdeconnect/

   # Fix if needed
   chmod 755 ~/.local/share/kdeconnect/
   ```

3. **Test with Small File**
   ```bash
   # Create test file
   echo "test" > /tmp/test.txt

   # Try sending
   kdeconnect-cli -d [device-id] --share /tmp/test.txt
   ```

4. **Check File Size**
   - Very large files (>2GB) may timeout
   - Try splitting large files
   - Use alternative for huge files (USB, cloud)

### Files Not Appearing in Downloads

**Problem**: File transfer succeeds but file can't be found.

**Solutions**:

1. **Check Download Location**
   ```bash
   # Default location
   ls -lh ~/.local/share/kdeconnect/

   # Also check
   ls -lh ~/Downloads/
   ```

2. **Search for File**
   ```bash
   find ~ -name "[filename]" -type f -mmin -10
   ```

3. **Check Logs**
   ```bash
   journalctl --user -u kdeconnect-daemon | grep -i "received file"
   ```

### Slow File Transfers

**Problem**: File transfers are very slow.

**Solutions**:

1. **Check Network Speed**
   ```bash
   # Install iperf3 on both devices
   # On desktop: iperf3 -s
   # On mobile: iperf3 -c [desktop-ip]
   ```

2. **WiFi Band**
   - Use 5GHz WiFi if available
   - 2.4GHz is slower but better range
   - Check router settings

3. **Router Position**
   - Move closer to router
   - Reduce obstacles between devices
   - Check signal strength

4. **Network Congestion**
   - Pause other downloads/uploads
   - Disconnect unused devices
   - Try at different time

## Plugin Problems

### Clipboard Not Syncing

**Problem**: Clipboard doesn't sync between devices.

**Solutions**:

1. **Verify Plugin Enabled**
   ```bash
   kdeconnect-cli -d [device-id] --info | grep clipboard
   ```

2. **Test Clipboard**
   ```bash
   # On desktop
   echo "test" | xclip -selection clipboard

   # Check if it appeared on mobile
   ```

3. **Restart Clipboard Manager**
   - Some clipboard managers conflict
   - Try disabling desktop clipboard manager
   - Test with basic clipboard

4. **Check Permissions**
   - Android: Accessibility permissions for KDE Connect
   - Settings → Accessibility → KDE Connect

### Notifications Not Syncing

**Problem**: Phone notifications don't appear on desktop.

**Solutions**:

1. **Check Plugin Status**
   - Desktop: Plugin enabled in applet
   - Mobile: Plugin enabled in app

2. **Verify Notification Access**
   - Android: Settings → Apps → KDE Connect → Notification access
   - Grant permission if not already granted

3. **Test Notification**
   - Send yourself a test message
   - Check if it appears on desktop

4. **Check Filter Settings**
   - Mobile app might be filtering notifications
   - KDE Connect → Device → Plugins → Notification sync
   - Check application filters

### Battery Status Not Showing

**Problem**: Can't see phone battery level on desktop.

**Solutions**:

1. **Enable Plugin**
   ```bash
   kdeconnect-cli -d [device-id] --info | grep battery
   ```

2. **Check Mobile Permissions**
   - Some Android versions require battery permission
   - Settings → Apps → KDE Connect → Permissions

3. **Wait for Update**
   - Battery level updates every few minutes
   - Not real-time

### Media Control Not Working

**Problem**: Can't control desktop media players from phone.

**Solutions**:

1. **Check Player Compatibility**
   ```bash
   # List MPRIS-compatible players
   dbus-send --print-reply --dest=org.freedesktop.DBus \
     /org/freedesktop/DBus org.freedesktop.DBus.ListNames | \
     grep mpris
   ```

2. **Test MPRIS**
   ```bash
   # Play something in browser or music player
   # Check if it appears
   playerctl -l
   ```

3. **Install mpris2 Support**
   - Browser: Install browser extension
   - VLC: MPRIS enabled by default
   - Spotify: Native MPRIS support

4. **Plugin Settings**
   - Desktop: Enable MPRIS plugin
   - Mobile: Enable "Multimedia control receiver"

## Performance Issues

### High CPU Usage

**Problem**: kdeconnect-daemon using excessive CPU.

**Solutions**:

1. **Check Running Plugins**
   ```bash
   kdeconnect-cli -d [device-id] --info
   ```

2. **Disable Unused Plugins**
   - Each plugin consumes resources
   - Disable features you don't use

3. **Check for Loops**
   ```bash
   # Look for repeated errors
   journalctl --user -u kdeconnect-daemon -n 100
   ```

4. **Restart Daemon**
   ```bash
   systemctl --user restart kdeconnect-daemon
   ```

### High Memory Usage

**Problem**: Daemon consuming too much RAM.

**Solutions**:

1. **Check Memory Usage**
   ```bash
   ps aux | grep kdeconnect-daemon
   ```

2. **Clear Cache**
   ```bash
   systemctl --user stop kdeconnect-daemon
   rm -rf ~/.cache/kdeconnect/
   systemctl --user start kdeconnect-daemon
   ```

3. **Limit File Transfers**
   - Large file transfers use memory
   - Transfer one file at a time

### Mobile Battery Drain

**Problem**: KDE Connect draining phone battery quickly.

**Solutions**:

1. **Disable Continuous Plugins**
   - Notification sync
   - Clipboard sync
   - These run constantly

2. **Reduce Update Frequency**
   - Check plugin settings
   - Increase poll intervals

3. **Disconnect When Not Needed**
   - Unpair when away from desktop
   - Or disable WiFi

4. **Check Background Activity**
   - Android: Settings → Apps → KDE Connect → Battery
   - View background activity
   - Restrict if excessive

## System-Specific Issues

### NixOS Issues

**Problem**: Can't build or install on NixOS.

**Solutions**:

1. **Use Flake**
   ```nix
   {
     inputs.cosmic-applet-kdeconnect.url = "github:olafkfreund/cosmic-applet-kdeconnect";

     environment.systemPackages = [
       inputs.cosmic-applet-kdeconnect.packages.${system}.default
     ];
   }
   ```

2. **Firewall Configuration**
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

3. **Missing Dependencies**
   ```bash
   nix develop
   ```

### Ubuntu/Debian Issues

**Problem**: Missing system libraries.

**Solutions**:

1. **Install Development Headers**
   ```bash
   sudo apt install \
     libxkbcommon-dev \
     libwayland-dev \
     libdbus-1-dev \
     libssl-dev \
     pkg-config
   ```

2. **Update System**
   ```bash
   sudo apt update
   sudo apt upgrade
   ```

### Arch Linux Issues

**Problem**: Build failures or dependency conflicts.

**Solutions**:

1. **Update System**
   ```bash
   sudo pacman -Syu
   ```

2. **Install Build Dependencies**
   ```bash
   sudo pacman -S base-devel rust cargo
   ```

3. **Clear Build Cache**
   ```bash
   cargo clean
   just build
   ```

### Fedora Issues

**Problem**: SELinux blocking connections.

**Solutions**:

1. **Check SELinux**
   ```bash
   getenforce
   ```

2. **Temporarily Disable** (Testing Only)
   ```bash
   sudo setenforce 0
   ```

3. **Create SELinux Policy**
   ```bash
   # Check denials
   sudo ausearch -m avc -ts recent | grep kdeconnect

   # Generate policy
   sudo ausearch -m avc -ts recent | grep kdeconnect | audit2allow -M kdeconnect

   # Install policy
   sudo semodule -i kdeconnect.pp
   ```

## Error Messages

### "Failed to create identity packet"

**Cause**: Configuration file corrupted or missing.

**Solution**:
```bash
systemctl --user stop kdeconnect-daemon
rm ~/.config/kdeconnect/config
systemctl --user start kdeconnect-daemon
```

### "TLS handshake failed"

**Cause**: Certificate mismatch or expired.

**Solution**:
```bash
# Regenerate certificates
systemctl --user stop kdeconnect-daemon
rm -rf ~/.config/kdeconnect/*/certificate.pem
rm -rf ~/.config/kdeconnect/*/privateKey.pem
systemctl --user start kdeconnect-daemon

# Re-pair devices
```

### "Port already in use"

**Cause**: Another instance or application using KDE Connect ports.

**Solution**:
```bash
# Find what's using the port
sudo netstat -tulpn | grep 1716

# Kill old instance
killall kdeconnect-daemon

# Start fresh
systemctl --user start kdeconnect-daemon
```

### "Permission denied" for file transfer

**Cause**: Insufficient permissions for download folder.

**Solution**:
```bash
# Fix permissions
chmod 755 ~/.local/share/kdeconnect/
mkdir -p ~/.local/share/kdeconnect/downloads/
chmod 755 ~/.local/share/kdeconnect/downloads/
```

### "Device not trusted"

**Cause**: Pairing incomplete or certificate invalid.

**Solution**:
```bash
# Unpair and re-pair
kdeconnect-cli -d [device-id] --unpair
# Then pair again through UI
```

## Debug Mode

### Enable Verbose Logging

```bash
# Stop daemon
systemctl --user stop kdeconnect-daemon

# Start with debug output
QT_LOGGING_RULES="kdeconnect.*=true" kdeconnect-daemon

# Or set permanently
export QT_LOGGING_RULES="kdeconnect.*=true"
systemctl --user start kdeconnect-daemon
```

### View Real-Time Logs

```bash
# Follow daemon logs
journalctl --user -u kdeconnect-daemon -f

# With debug output
journalctl --user -u kdeconnect-daemon -f --output=verbose
```

### Capture Logs for Bug Reports

```bash
# Capture last 100 lines
journalctl --user -u kdeconnect-daemon -n 100 > kdeconnect-debug.log

# Capture with timestamps
journalctl --user -u kdeconnect-daemon --since "10 minutes ago" > debug.log
```

### Network Packet Capture

```bash
# Install tcpdump
sudo apt install tcpdump  # Ubuntu/Debian
sudo pacman -S tcpdump    # Arch
sudo dnf install tcpdump  # Fedora

# Capture KDE Connect traffic
sudo tcpdump -i any -w kdeconnect.pcap port 1716
```

### Test Individual Components

```bash
# Test device discovery
kdeconnect-cli -l --refresh

# Test specific device
kdeconnect-cli -d [device-id] --info

# Test ping
kdeconnect-cli -d [device-id] --ping

# Test file share
kdeconnect-cli -d [device-id] --share /tmp/test.txt
```

## Reset to Factory State

### Complete Reset (Desktop)

```bash
# Stop all services
systemctl --user stop kdeconnect-daemon

# Remove all data
rm -rf ~/.config/kdeconnect/
rm -rf ~/.local/share/kdeconnect/
rm -rf ~/.cache/kdeconnect/

# Restart daemon
systemctl --user start kdeconnect-daemon
```

### Complete Reset (Mobile)

1. Open KDE Connect app
2. Settings → Advanced
3. **"Clear app data"** or uninstall/reinstall

### Selective Reset

```bash
# Reset only one device
systemctl --user stop kdeconnect-daemon
rm -rf ~/.config/kdeconnect/[device-id]/
systemctl --user start kdeconnect-daemon

# Reset only certificates
rm -rf ~/.config/kdeconnect/*/certificate.pem
rm -rf ~/.config/kdeconnect/*/privateKey.pem

# Reset only received files
rm -rf ~/.local/share/kdeconnect/downloads/
```

## Getting Help

### Before Asking for Help

1. **Check logs**:
   ```bash
   journalctl --user -u kdeconnect-daemon -n 50
   ```

2. **Gather system info**:
   ```bash
   # OS version
   cat /etc/os-release

   # Kernel version
   uname -a

   # KDE Connect version
   kdeconnect-daemon --version

   # Device list
   kdeconnect-cli -l
   ```

3. **Try basic troubleshooting**:
   - Restart daemon
   - Check firewall
   - Verify same network
   - Test with ping

### Where to Get Help

1. **Documentation**
   - [USER_GUIDE.md](USER_GUIDE.md)
   - [INSTALL.md](INSTALL.md)
   - [GitHub README](https://github.com/olafkfreund/cosmic-applet-kdeconnect)

2. **Community Support**
   - [GitHub Issues](https://github.com/olafkfreund/cosmic-applet-kdeconnect/issues)
   - [COSMIC Desktop Chat](https://chat.pop-os.org/)
   - [KDE Connect Forum](https://forum.kde.org/)

3. **Bug Reports**
   - Check existing issues first
   - Include logs and system info
   - Describe steps to reproduce
   - Note expected vs actual behavior

### Information to Include in Bug Reports

```markdown
**System Information:**
- OS: [NixOS 24.11 / Ubuntu 24.04 / etc]
- Kernel: [uname -r output]
- Desktop: COSMIC Alpha 2
- KDE Connect Version: [version]

**Mobile Device:**
- OS: [Android 14 / iOS 17 / etc]
- KDE Connect App Version: [version]
- Device Model: [model]

**Network:**
- Connection Type: WiFi / Ethernet
- Same Network: Yes / No
- Firewall: [firewalld / ufw / none]

**Problem Description:**
[Clear description of the issue]

**Steps to Reproduce:**
1. [Step 1]
2. [Step 2]
3. [Step 3]

**Expected Behavior:**
[What should happen]

**Actual Behavior:**
[What actually happens]

**Logs:**
```
[Relevant log output]
```

**Additional Context:**
[Any other relevant information]
```

## Known Issues

### Issue: Bluetooth Support

**Status**: Planned feature, not yet implemented

**Workaround**: Use WiFi connection only

### Issue: iOS Limitations

**Status**: iOS app has fewer features than Android

**Affected**:
- Limited background operation
- Some plugins unavailable

**Workaround**: Use Android device for full feature set

### Issue: Large File Performance

**Status**: Transfers >2GB may be slow or timeout

**Workaround**:
- Use USB cable for very large files
- Split large files
- Use cloud storage alternative

### Issue: COSMIC Alpha Compatibility

**Status**: COSMIC Desktop is in alpha, APIs may change

**Workaround**: Keep applet updated with latest COSMIC

---

**Last Updated**: 2026-01-13
**Version**: 1.0.0

For installation instructions, see [INSTALL.md](INSTALL.md)
For usage guide, see [USER_GUIDE.md](USER_GUIDE.md)
