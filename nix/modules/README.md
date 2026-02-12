# COSMIC Connect NixOS Modules

This directory contains modular NixOS configurations for COSMIC Connect features.

## PAM Phone Authentication Module

The `pam-cosmic-connect.nix` module enables phone-based authentication for PAM services.

### Quick Start

Add to your NixOS configuration:

```nix
{
  imports = [
    # Import the COSMIC Connect module
    ./path/to/cosmic-connect/nix/module.nix
  ];

  # Enable COSMIC Connect
  services.cosmic-connect = {
    enable = true;
    daemon.enable = true;
  };

  # Enable phone authentication for PAM
  security.pam.services.cosmic-connect = {
    enable = true;
    timeout = 30;
    services = [ "login" "sudo" "cosmic-greeter" ];
    fallbackToPassword = true;
  };
}
```

### Configuration Options

#### `security.pam.services.cosmic-connect.enable`

**Type:** `boolean`
**Default:** `false`

Enable COSMIC Connect phone authentication for PAM services.

#### `security.pam.services.cosmic-connect.timeout`

**Type:** `integer`
**Default:** `30`

Timeout in seconds for phone authentication requests. If the phone does not respond within this time, authentication falls back to password (if enabled).

#### `security.pam.services.cosmic-connect.services`

**Type:** `list of strings`
**Default:** `[ "login" "sudo" "cosmic-greeter" ]`

List of PAM services to enable phone authentication for. Common services:

- `"login"` - Console login
- `"sudo"` - Privilege escalation
- `"cosmic-greeter"` - Display manager login
- `"polkit-1"` - PolicyKit authentication dialogs

#### `security.pam.services.cosmic-connect.fallbackToPassword`

**Type:** `boolean`
**Default:** `true`

Whether to fall back to password authentication if phone authentication fails.

**WARNING:** Disabling this is NOT recommended as it may lock you out of your system if your phone is unavailable.

### Usage Examples

#### Minimal Setup

Enable phone authentication with defaults:

```nix
security.pam.services.cosmic-connect.enable = true;
```

#### Custom Timeout

Increase timeout for slower network connections:

```nix
security.pam.services.cosmic-connect = {
  enable = true;
  timeout = 60;
};
```

#### Additional Services

Enable phone auth for more PAM services:

```nix
security.pam.services.cosmic-connect = {
  enable = true;
  services = [
    "login"
    "sudo"
    "cosmic-greeter"
    "polkit-1"
    "systemd-user"
  ];
};
```

#### Advanced Security (Not Recommended)

Disable password fallback (requires phone for all authentication):

```nix
security.pam.services.cosmic-connect = {
  enable = true;
  fallbackToPassword = false;  # DANGEROUS!
};
```

**NOTE:** This configuration will display a warning during `nixos-rebuild` and may lock you out if your phone is unavailable.

### PAM Stack Behavior

The generated PAM configuration looks like this:

```pam
# /etc/pam.d/sudo (example)
# COSMIC Connect phone authentication
auth  [success=done default=ignore]  pam_cosmic_connect.so timeout=30

# Standard password authentication (fallback)
auth  required  pam_unix.so try_first_pass
```

**Flow:**
1. PAM requests authentication
2. `pam_cosmic_connect.so` sends request to paired phone via D-Bus
3. User approves/denies on phone within timeout
4. On success (`[success=done]`): Authentication complete, skip remaining modules
5. On failure/timeout (`[default=ignore]`): Continue to password authentication

### Requirements

- COSMIC Connect daemon must be running (`services.cosmic-connect.daemon.enable = true`)
- At least one phone must be paired with the desktop
- D-Bus session bus must be available
- Network connectivity between devices

### Troubleshooting

#### Phone authentication not working

1. Check daemon status:
   ```bash
   systemctl --user status cosmic-ext-connect-daemon
   ```

2. Verify paired devices:
   ```bash
   cosmic-connect-cli devices
   ```

3. Check PAM configuration:
   ```bash
   cat /etc/pam.d/sudo
   ```

4. View daemon logs:
   ```bash
   journalctl --user -u cosmic-ext-connect-daemon -f
   ```

#### Locked out of system

If you're locked out due to phone authentication issues:

1. Boot into recovery mode
2. Edit `/etc/nixos/configuration.nix`
3. Set `security.pam.services.cosmic-connect.enable = false;`
4. Rebuild: `nixos-rebuild switch`

### Security Considerations

- **Phone Availability:** Ensure your phone is charged and within network range
- **Backup Authentication:** Always keep password fallback enabled (`fallbackToPassword = true`)
- **Network Security:** Phone authentication uses TLS-encrypted communication
- **Trust Model:** Initial device pairing requires physical confirmation on both devices

### Related Modules

- Main module: `../module.nix`
- Package definition: `../package.nix`

### Contributing

See the main [COSMIC Connect repository](https://github.com/olafkfreund/cosmic-connect-desktop-app) for contribution guidelines.
