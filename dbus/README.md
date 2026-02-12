# D-Bus Interface Definitions

This directory contains D-Bus interface definitions for COSMIC Connect services.

## Files

### io.github.olafkfreund.CosmicExtPhoneAuth.xml
D-Bus interface definition for phone-based biometric authentication. Defines methods, signals, and properties for the authentication service.

### io.github.olafkfreund.CosmicExtPhoneAuth.service
D-Bus service activation file. Tells D-Bus how to start the authentication service.

### io.github.olafkfreund.CosmicExtPhoneAuth.conf
D-Bus security policy configuration. Controls who can access the authentication service.

### polkit/io.github.olafkfreund.CosmicExtPhoneAuth.policy
Polkit authorization rules. Defines privilege requirements for administrative actions.

## Installation

### For Development (Session Bus)

```bash
# Install D-Bus service file
mkdir -p ~/.local/share/dbus-1/services
cp io.github.olafkfreund.CosmicExtPhoneAuth.service ~/.local/share/dbus-1/services/

# Install D-Bus configuration
mkdir -p ~/.local/share/dbus-1/session.d
cp io.github.olafkfreund.CosmicExtPhoneAuth.conf ~/.local/share/dbus-1/session.d/

# Install Polkit policy
sudo cp polkit/io.github.olafkfreund.CosmicExtPhoneAuth.policy /usr/share/polkit-1/actions/
```

### For System-Wide Installation

```bash
# Install D-Bus service file
sudo cp io.github.olafkfreund.CosmicExtPhoneAuth.service /usr/share/dbus-1/services/

# Install D-Bus configuration
sudo cp io.github.olafkfreund.CosmicExtPhoneAuth.conf /usr/share/dbus-1/session.d/

# Install Polkit policy
sudo cp polkit/io.github.olafkfreund.CosmicExtPhoneAuth.policy /usr/share/polkit-1/actions/

# Reload Polkit
sudo systemctl reload polkit
```

## Validation

### Validate XML Interface

```bash
# Check XML syntax
xmllint --noout io.github.olafkfreund.CosmicExtPhoneAuth.xml

# Validate against D-Bus DTD
xmllint --valid --noout io.github.olafkfreund.CosmicExtPhoneAuth.xml
```

### Test D-Bus Configuration

```bash
# Check D-Bus configuration syntax
dbus-daemon --config-file=io.github.olafkfreund.CosmicExtPhoneAuth.conf --print-address --nofork --session
```

### Verify Polkit Policy

```bash
# List available actions
pkaction | grep io.github.olafkfreund.CosmicExtPhoneAuth

# Check authorization for current user
pkcheck --action-id io.github.olafkfreund.CosmicExtPhoneAuth.request --process $$
pkcheck --action-id io.github.olafkfreund.CosmicExtPhoneAuth.admin --process $$
```

## Introspection (After Service is Running)

```bash
# Introspect the interface
gdbus introspect --session \
  --dest io.github.olafkfreund.CosmicExtPhoneAuth \
  --object-path /io/github/olafkfreund/CosmicExtPhoneAuth

# Call a method (example)
gdbus call --session \
  --dest io.github.olafkfreund.CosmicExtPhoneAuth \
  --object-path /io/github/olafkfreund/CosmicExtPhoneAuth \
  --method io.github.olafkfreund.CosmicExtPhoneAuth.RequestAuth "username" "sudo"

# Monitor signals
gdbus monitor --session --dest io.github.olafkfreund.CosmicExtPhoneAuth

# Get properties
gdbus call --session \
  --dest io.github.olafkfreund.CosmicExtPhoneAuth \
  --object-path /io/github/olafkfreund/CosmicExtPhoneAuth \
  --method org.freedesktop.DBus.Properties.Get \
  "io.github.olafkfreund.CosmicExtPhoneAuth" "ConnectedDevices"
```

## Testing with d-feet

The [d-feet](https://wiki.gnome.org/Apps/DFeet) GUI tool provides an easy way to explore and test D-Bus interfaces:

```bash
# Install d-feet
sudo apt install d-feet  # Ubuntu/Debian
sudo dnf install d-feet  # Fedora

# Launch d-feet and connect to Session Bus
d-feet
```

## Interface Overview

### Methods

- **RequestAuth(username, auth_type) → request_id**
  - Initiate a new authentication request
  - Returns a unique ID for tracking

- **CheckAuth(request_id) → (approved, biometric_type)**
  - Check the status of a pending request
  - Returns approval status and biometric method used

- **CancelAuth(request_id) → success**
  - Cancel a pending authentication request
  - Returns true if successfully cancelled

- **GetPendingRequests() → requests**
  - Administrative method to list all pending requests
  - Requires Polkit `io.github.olafkfreund.CosmicExtPhoneAuth.admin` authorization

### Signals

- **AuthCompleted(request_id, approved, biometric_type)**
  - Emitted when an authentication request completes
  - PAM modules should listen for this signal

- **AuthTimeout(request_id)**
  - Emitted when a request times out
  - Treated as authentication failure

### Properties

- **DefaultTimeout** (uint64, read-only)
  - Default timeout in seconds for auth requests
  - Default: 30 seconds

- **ConnectedDevices** (uint32, read-only)
  - Number of connected devices capable of auth
  - If 0, RequestAuth will fail immediately

## Security Considerations

### Authentication Flow

1. PAM module calls `RequestAuth()` with username and auth type
2. Service sends request to paired mobile devices
3. User approves/denies on their phone using biometrics
4. Service emits `AuthCompleted` signal
5. PAM module receives signal and grants/denies access

### Authorization Levels

- **Regular users** can:
  - Request authentication for their own account
  - Check status of their own requests
  - Cancel their own requests
  - Receive authentication signals

- **Administrators** can:
  - View all pending requests across all users
  - Configure system-wide authentication settings

### Polkit Actions

- `io.github.olafkfreund.CosmicExtPhoneAuth.request` - Request authentication (active user)
- `io.github.olafkfreund.CosmicExtPhoneAuth.cancel` - Cancel own requests (active user)
- `io.github.olafkfreund.CosmicExtPhoneAuth.admin` - View all requests (admin)
- `io.github.olafkfreund.CosmicExtPhoneAuth.configure` - Modify settings (admin)

## Implementation Notes

The actual D-Bus service implementation will be in `cosmic-ext-connect-daemon` using the `zbus` Rust library. The interface definition here serves as the contract between the daemon and PAM modules.

### Object Path

The service will be available at object path: `/io/github/olafkfreund/CosmicExtPhoneAuth`

### Bus Type

The service runs on the **session bus** (not system bus) to maintain user isolation and follow COSMIC Desktop patterns.

## Related Files

- `cosmic-ext-connect-daemon/src/dbus.rs` - Existing D-Bus implementation
- `io.github.olafkfreund.CosmicExtConnect.service` - Main service D-Bus activation
- PAM module implementation (to be created in Phase 2)

## References

- [D-Bus Specification](https://dbus.freedesktop.org/doc/dbus-specification.html)
- [Polkit Authorization](https://www.freedesktop.org/software/polkit/docs/latest/)
- [zbus Documentation](https://docs.rs/zbus/)
- [PAM Programming Guide](http://www.linux-pam.org/Linux-PAM-html/)
